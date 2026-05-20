//! Global shortcut via `org.freedesktop.portal.GlobalShortcuts` (Wayland; the
//! `tauri-plugin-global-shortcut` stack is X11-only on Linux).

use std::thread::JoinHandle;
use std::sync::{Mutex, OnceLock, mpsc as std_mpsc};

use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use ashpd::desktop::CreateSessionOptions;
use futures_util::StreamExt;
use tauri::AppHandle;
use tokio::sync::oneshot;

const PORTAL_ID: &str = "vcm_main_toggle";

struct ActivePortal {
    stop: oneshot::Sender<()>,
    join: JoinHandle<()>,
}

static PORTAL: OnceLock<Mutex<Option<ActivePortal>>> = OnceLock::new();

fn state() -> &'static Mutex<Option<ActivePortal>> {
    PORTAL.get_or_init(|| Mutex::new(None))
}

pub fn is_wayland_session() -> bool {
    if std::env::var("WAYLAND_DISPLAY")
        .map(|d| !d.is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    if let Ok(t) = std::env::var("XDG_SESSION_TYPE") {
        t.eq_ignore_ascii_case("wayland")
    } else {
        false
    }
}

/// <https://specifications.freedesktop.org/shortcuts-spec/latest/shortcuts.html>
fn to_xdg_shortcut_trigger(s: &str) -> String {
    let parts: Vec<&str> = s
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();
    let mut out: Vec<String> = Vec::new();
    for p in &parts {
        let u = p.to_uppercase();
        let seg: String = match u.as_str() {
            "CTRL" | "CONTROL" => "CTRL".into(),
            "SHIFT" => "SHIFT".into(),
            "ALT" => "ALT".into(),
            "SUPER" | "META" | "WIN" | "WINDOWS" | "WINDOW" | "CMD" | "COMMAND" => "LOGO".into(),
            _ if p.len() == 1 && p.chars().next().is_some_and(|c| c.is_alphabetic()) => {
                p.to_lowercase()
            }
            _ => p.to_string(),
        };
        out.push(seg);
    }
    out.join("+")
}

pub fn stop() {
    let old = { state().lock().unwrap().take() };
    if let Some(t) = old {
        let _ = t.stop.send(());
        let _ = t.join.join();
    }
}

/// `Ok(())` if the Global Shortcuts portal is handling the key; on `Err` use Tauri (X11 grab).
pub fn start(app: AppHandle, user_shortcut: &str) -> Result<(), String> {
    if !is_wayland_session() {
        return Err("not Wayland".into());
    }
    crate::config::validate_shortcut(user_shortcut)?;
    let trigger = to_xdg_shortcut_trigger(user_shortcut);
    let (stop_tx, stop_rx) = oneshot::channel();
    let (ready_tx, ready_rx) = std_mpsc::channel::<Result<(), String>>();

    let j = {
        let app = app.clone();
        let trigger = trigger.clone();
        let user_shortcut = user_shortcut.to_string();
        std::thread::Builder::new()
            .name("vcm-portal-globals".into())
            .spawn(move || {
                let Ok(rt) = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                else {
                    let _ = ready_tx.send(Err("tokio runtime init failed".into()));
                    return;
                };
                rt.block_on(portal_entry(
                    app, &user_shortcut, &trigger, ready_tx, stop_rx,
                ));
            })
            .map_err(|e| e.to_string())?
    };

    let bind_result = match ready_rx.recv() {
        Ok(x) => x,
        Err(e) => {
            let _ = j.join();
            return Err(format!("internal channel: {e}"));
        }
    };
    if let Err(e) = bind_result {
        let _ = j.join();
        return Err(e);
    }
    *state().lock().unwrap() = Some(ActivePortal { stop: stop_tx, join: j });
    Ok(())
}

async fn portal_entry(
    app: AppHandle,
    _user_display: &str,
    trigger: &str,
    ready_tx: std_mpsc::Sender<Result<(), String>>,
    mut stop_rx: oneshot::Receiver<()>,
) {
    let portal = match GlobalShortcuts::new().await {
        Ok(p) => p,
        Err(e) => {
            let _ = ready_tx.send(Err(format!("GlobalShortcuts::new: {e}")));
            return;
        }
    };
    let session = match portal
        .create_session(CreateSessionOptions::default())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            return;
        }
    };
    let sc = NewShortcut::new(PORTAL_ID, "Toggle Vibes Copy Manager").preferred_trigger(Some(trigger));
    let request = match portal
        .bind_shortcuts(
            &session,
            std::slice::from_ref(&sc),
            None,
            Default::default(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            let _ = session.close().await;
            return;
        }
    };
    let response = match request.response() {
        Ok(b) => b,
        Err(e) => {
            let _ = ready_tx.send(Err(e.to_string()));
            let _ = session.close().await;
            return;
        }
    };
    type PortalShortcut = ashpd::desktop::global_shortcuts::Shortcut;
    let has = response
        .shortcuts()
        .iter()
        .any(|sh: &PortalShortcut| sh.id() == PORTAL_ID);
    if !has {
        let _ = ready_tx.send(Err(
            "GlobalShortcuts: no shortcut was bound; allow it in the portal or pick another"
                .into(),
        ));
        let _ = session.close().await;
        return;
    }
    if ready_tx.send(Ok(())).is_err() {
        let _ = session.close().await;
        return;
    }
    let mut act = match portal.receive_activated().await {
        Ok(s) => s,
        Err(e) => {
            log::warn!("GlobalShortcuts::receive_activated: {e}");
            let _ = session.close().await;
            return;
        }
    };
    loop {
        tokio::select! {
            biased;
            _ = &mut stop_rx => break,
            next = act.next() => {
                let Some(activated) = next else { break; };
                if activated.shortcut_id() != PORTAL_ID {
                    continue;
                }
                let h = app.clone();
                if let Err(e) = h.run_on_main_thread({
                    let h2 = h.clone();
                    move || {
                        let _ = crate::window::do_toggle(&h2);
                    }
                }) {
                    log::warn!("run_on_main (portal shortcut): {e}");
                }
            }
        }
    }
    let _ = session.close().await;
}
