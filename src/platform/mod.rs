#[cfg(target_os = "linux")]
use std::fs;

/// Platform detection utilities
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detect the current operating system
    pub fn detect_os() -> OperatingSystem {
        #[cfg(target_os = "linux")]
        {
            OperatingSystem::Linux
        }
        #[cfg(target_os = "macos")]
        {
            OperatingSystem::MacOS
        }
        #[cfg(target_os = "windows")]
        {
            OperatingSystem::Windows
        }
        #[cfg(target_os = "freebsd")]
        {
            OperatingSystem::FreeBSD
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows",
            target_os = "freebsd"
        )))]
        {
            OperatingSystem::Unknown
        }
    }

    /// Detect the Linux distribution
    #[cfg(target_os = "linux")]
    pub fn detect_linux_distro() -> LinuxDistribution {
        // Check for specific distribution files
        if fs::metadata("/etc/arch-release").is_ok() {
            return LinuxDistribution::Arch;
        }
        if fs::metadata("/etc/debian_version").is_ok() {
            if fs::metadata("/etc/pop-release").is_ok() {
                return LinuxDistribution::PopOS;
            }
            // Check if it's Ubuntu
            if let Ok(contents) = fs::read_to_string("/etc/os-release") {
                if contents.contains("Ubuntu") {
                    return LinuxDistribution::Ubuntu;
                }
            }
            return LinuxDistribution::Debian;
        }
        if fs::metadata("/etc/fedora-release").is_ok() {
            return LinuxDistribution::Fedora;
        }
        if fs::metadata("/etc/redhat-release").is_ok() {
            return LinuxDistribution::RedHat;
        }
        if fs::metadata("/etc/SuSE-release").is_ok() || fs::metadata("/etc/SUSE-brand").is_ok() {
            return LinuxDistribution::OpenSUSE;
        }
        if fs::metadata("/etc/alpine-release").is_ok() {
            return LinuxDistribution::Alpine;
        }
        if fs::metadata("/etc/gentoo-release").is_ok() {
            return LinuxDistribution::Gentoo;
        }

        LinuxDistribution::Unknown
    }

    /// Get the recommended package manager for the current platform
    pub fn recommended_package_manager() -> Option<&'static str> {
        match Self::detect_os() {
            OperatingSystem::MacOS => Some("brew"),
            OperatingSystem::Windows => Some("winget"),
            OperatingSystem::FreeBSD => Some("pkg"),
            OperatingSystem::Linux => {
                #[cfg(target_os = "linux")]
                {
                    match Self::detect_linux_distro() {
                        LinuxDistribution::Arch => Some("pacman"),
                        LinuxDistribution::Debian
                        | LinuxDistribution::Ubuntu
                        | LinuxDistribution::PopOS => Some("apt"),
                        LinuxDistribution::Fedora => Some("dnf"),
                        LinuxDistribution::RedHat => Some("yum"),
                        LinuxDistribution::OpenSUSE => Some("zypper"),
                        LinuxDistribution::Alpine => Some("apk"),
                        LinuxDistribution::Gentoo => Some("emerge"),
                        LinuxDistribution::Unknown => None,
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    None
                }
            }
            OperatingSystem::Unknown => None,
        }
    }

    /// Check if the current distribution is a rolling release
    #[cfg(target_os = "linux")]
    pub fn is_rolling_release() -> bool {
        match Self::detect_linux_distro() {
            LinuxDistribution::Arch => true,
            LinuxDistribution::Gentoo => true,
            LinuxDistribution::OpenSUSE => {
                if let Ok(contents) = fs::read_to_string("/etc/os-release") {
                    contents.contains("Tumbleweed")
                } else {
                    false
                }
            }
            LinuxDistribution::Debian => {
                if let Ok(contents) = fs::read_to_string("/etc/os-release") {
                    contents.contains("Debian GNU/Linux testing")
                        || contents.contains("Debian GNU/Linux unstable")
                        || contents.contains("Debian GNU/Linux sid")
                } else {
                    false
                }
            }
            LinuxDistribution::Fedora => {
                if let Ok(contents) = fs::read_to_string("/etc/os-release") {
                    contents.contains("Rawhide")
                } else {
                    false
                }
            }
            LinuxDistribution::Ubuntu
            | LinuxDistribution::PopOS
            | LinuxDistribution::RedHat
            | LinuxDistribution::Alpine
            | LinuxDistribution::Unknown => false,
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn is_rolling_release() -> bool {
        false
    }

    /// Get a human-readable description of the current platform
    pub fn platform_description() -> String {
        match Self::detect_os() {
            OperatingSystem::Linux => {
                #[cfg(target_os = "linux")]
                {
                    format!("{} ({})", Self::detect_linux_distro(), Self::detect_os())
                }
                #[cfg(not(target_os = "linux"))]
                {
                    Self::detect_os().to_string()
                }
            }
            os => os.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
    FreeBSD,
    Unknown,
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OperatingSystem::Linux => "Linux",
            OperatingSystem::MacOS => "macOS",
            OperatingSystem::Windows => "Windows",
            OperatingSystem::FreeBSD => "FreeBSD",
            OperatingSystem::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(target_os = "linux")]
pub enum LinuxDistribution {
    Arch,
    Debian,
    Ubuntu,
    PopOS,
    Fedora,
    RedHat,
    OpenSUSE,
    Alpine,
    Gentoo,
    Unknown,
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for LinuxDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LinuxDistribution::Arch => "Arch Linux",
            LinuxDistribution::Debian => "Debian",
            LinuxDistribution::Ubuntu => "Ubuntu",
            LinuxDistribution::PopOS => "Pop!_OS",
            LinuxDistribution::Fedora => "Fedora",
            LinuxDistribution::RedHat => "Red Hat",
            LinuxDistribution::OpenSUSE => "openSUSE",
            LinuxDistribution::Alpine => "Alpine Linux",
            LinuxDistribution::Gentoo => "Gentoo",
            LinuxDistribution::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = PlatformDetector::detect_os();
        assert_ne!(os, OperatingSystem::Unknown); // Should detect something
    }

    #[test]
    fn test_recommended_package_manager() {
        let pm = PlatformDetector::recommended_package_manager();
        // Should recommend something on most platforms
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        assert!(pm.is_some());
    }

    #[test]
    fn test_operating_system_variants() {
        let os = PlatformDetector::detect_os();
        match os {
            OperatingSystem::Linux
            | OperatingSystem::MacOS
            | OperatingSystem::Windows
            | OperatingSystem::FreeBSD
            | OperatingSystem::Unknown => {}
        }
        assert_ne!(os, OperatingSystem::Unknown);
    }

    #[test]
    fn test_operating_system_equality() {
        let os1 = OperatingSystem::Linux;
        let os2 = OperatingSystem::Linux;
        let os3 = OperatingSystem::MacOS;
        assert_eq!(os1, os2);
        assert_ne!(os1, os3);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_distribution_variants() {
        let distro = PlatformDetector::detect_linux_distro();
        match distro {
            LinuxDistribution::Arch
            | LinuxDistribution::Debian
            | LinuxDistribution::Ubuntu
            | LinuxDistribution::PopOS
            | LinuxDistribution::Fedora
            | LinuxDistribution::RedHat
            | LinuxDistribution::OpenSUSE
            | LinuxDistribution::Alpine
            | LinuxDistribution::Gentoo
            | LinuxDistribution::Unknown => {}
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_distribution_equality() {
        let distro1 = LinuxDistribution::Arch;
        let distro2 = LinuxDistribution::Arch;
        let distro3 = LinuxDistribution::Debian;
        assert_eq!(distro1, distro2);
        assert_ne!(distro1, distro3);
    }

    #[test]
    fn test_recommended_package_manager_consistency() {
        let os = PlatformDetector::detect_os();
        let pm = PlatformDetector::recommended_package_manager();

        match os {
            OperatingSystem::MacOS => assert_eq!(pm, Some("brew")),
            OperatingSystem::Windows => assert_eq!(pm, Some("winget")),
            OperatingSystem::FreeBSD => assert_eq!(pm, Some("pkg")),
            OperatingSystem::Linux => {
                #[cfg(target_os = "linux")]
                {
                    let distro = PlatformDetector::detect_linux_distro();
                    match distro {
                        LinuxDistribution::Arch => assert_eq!(pm, Some("pacman")),
                        LinuxDistribution::Debian
                        | LinuxDistribution::Ubuntu
                        | LinuxDistribution::PopOS => assert_eq!(pm, Some("apt")),
                        LinuxDistribution::Fedora => assert_eq!(pm, Some("dnf")),
                        LinuxDistribution::RedHat => assert_eq!(pm, Some("yum")),
                        LinuxDistribution::OpenSUSE => assert_eq!(pm, Some("zypper")),
                        LinuxDistribution::Alpine => assert_eq!(pm, Some("apk")),
                        LinuxDistribution::Gentoo => assert_eq!(pm, Some("emerge")),
                        LinuxDistribution::Unknown => assert_eq!(pm, None),
                    }
                }
            }
            OperatingSystem::Unknown => assert_eq!(pm, None),
        }
    }
}
