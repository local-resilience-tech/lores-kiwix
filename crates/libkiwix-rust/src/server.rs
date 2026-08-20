use crate::ffi;

/// Server address configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: i32,
    pub ip_mode: IpMode,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".into(),
            port: 8080,
            ip_mode: IpMode::Auto,
        }
    }
}

/// IP protocol selection for the server.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IpMode {
    Ipv4 = 0,
    Ipv6 = 1,
    All = 2,
    Auto = 3,
}

impl IpMode {
    pub(crate) fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Create and configure a Kiwix server.
pub fn new_server(library: crate::Library, config: &ServerConfig) -> crate::Server {
    let mut server = ffi::create_server(library);
    // SAFETY: `Server` is an opaque C++ type; these setters are non-const
    // member functions documented as safe to call via `pin_mut_unchecked`.
    unsafe {
        ffi::server_set_address(server.pin_mut_unchecked(), &config.address);
        ffi::server_set_port(server.pin_mut_unchecked(), config.port);
        ffi::server_set_ip_mode(server.pin_mut_unchecked(), config.ip_mode.as_i32());
    }
    server
}

/// Start the server.
pub fn server_start(server: &mut crate::Server) -> bool {
    // SAFETY: `Server` is an opaque C++ type; `start` is a non-const member.
    unsafe { ffi::server_start(server.pin_mut_unchecked()) }
}

/// Stop the server.
pub fn server_stop(server: &mut crate::Server) {
    // SAFETY: `Server` is an opaque C++ type; `stop` is a non-const member.
    unsafe { ffi::server_stop(server.pin_mut_unchecked()) };
}
