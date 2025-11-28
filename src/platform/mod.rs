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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
    FreeBSD,
    Unknown,
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
}
