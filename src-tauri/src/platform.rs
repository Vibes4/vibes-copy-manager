//! Platform detection utilities for display backend and environment.

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDisplayBackend {
    X11,
    Wayland,
    Unknown,
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for LinuxDisplayBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X11 => write!(f, "X11"),
            Self::Wayland => write!(f, "Wayland"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(target_os = "linux")]
impl LinuxDisplayBackend {
    /// Detect the active display backend from environment variables.
    pub fn detect() -> Self {
        // Check XDG_SESSION_TYPE first (most reliable)
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            match session_type.to_lowercase().as_str() {
                "wayland" => return Self::Wayland,
                "x11" => return Self::X11,
                _ => {}
            }
        }

        // Check for Wayland display socket
        if std::env::var("WAYLAND_DISPLAY")
            .map(|d| !d.is_empty())
            .unwrap_or(false)
        {
            return Self::Wayland;
        }

        // Check for X11 DISPLAY variable
        if std::env::var("DISPLAY")
            .map(|d| !d.is_empty())
            .unwrap_or(false)
        {
            return Self::X11;
        }

        Self::Unknown
    }

    pub fn is_wayland(self) -> bool {
        self == Self::Wayland
    }

    pub fn is_x11(self) -> bool {
        self == Self::X11
    }
}

/// Returns a human-readable string describing the current platform for logging.
pub fn platform_info() -> String {
    #[cfg(target_os = "linux")]
    {
        let backend = LinuxDisplayBackend::detect();
        format!("Linux ({})", backend)
    }
    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "Unknown OS".to_string()
    }
}
