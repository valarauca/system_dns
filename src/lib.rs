

#[cfg(any(unix))]
extern crate libc;
use libc::{
    getaddrinfo,
    addrinfo,
    sockaddr,
    c_char,
    freeaddrinfo,
    getnameinfo,
    gai_strerror,
    AF_INET,
    AF_INET6,
    NI_NUMERICHOST,
};
use std::str::FromStr;
use std::ffi::{
    CStr,
    CString,
};
use std::mem::{
    forget,
    transmute,
    zeroed,
};
use std::net::{
    Ipv4Addr,
    Ipv6Addr,
    IpAddr,
};

/// Resolve a host in ipv4
pub fn system_resolve_ipv4(host: &str)
    -> Result<Vec<IpAddr>,String>
{
    let addr = IpAddr::V4(Ipv4Addr::new(127,0,0,1));
    dry_resolve(host, &addr)
}

/// Resolve a host in ipv4
pub fn system_resolve_ipv6(host: &str)
    -> Result<Vec<IpAddr>,String>
{
    let addr = IpAddr::V6(Ipv6Addr::new(0,0,0,0,0,0,0,1));
    dry_resolve(host, &addr)
}

/// Convert Rust's `IpAddr` into the AF_INET representation
fn build_ip_type(x: &IpAddr) -> i32 {
    match x {
        &IpAddr::V4(_) => AF_INET,
        &IpAddr::V6(_) => AF_INET6,
    }
}

/// Create the dummy addrinfo to hint what type of address we're querying
unsafe fn build_hints(ip: &IpAddr) -> addrinfo {
    addrinfo {
        ai_flags: 0,
        ai_family: build_ip_type(ip),
        ai_socktype: 0,
        ai_protocol: 0,
        ai_addrlen: 0,
        ai_addr: zeroed(),
        ai_canonname: zeroed(),
        ai_next: zeroed(),
    }
}

/// convert error code to text
unsafe fn build_error(err: i32)
    -> String
{
    CStr::from_ptr(gai_strerror(err))
        .to_string_lossy()
        .to_string()
}

/// converts the on stack buffer to an ipaddress
unsafe fn to_ip(data: &[i8;100])
    -> Option<IpAddr>
{
    CStr::from_ptr(data.as_ptr())
        .to_str()
        .ok()
        .iter()
        .filter_map(|s| IpAddr::from_str(s).ok())
        .next()
}

fn dry_resolve(host: &str, kind: &IpAddr)
    -> Result<Vec<IpAddr>,String>
{
    let host = match CString::new(host) {
        Ok(x) => x,
        Err(_) => return Err("Null terminator in host name".to_string())
    };
    let mut addr = Vec::new();
    unsafe {

        // set up call to getaddrinfo
        let hints = build_hints(kind);
        let mut list: *mut addrinfo = zeroed();
        let host: *const c_char = transmute(host.as_bytes().as_ptr());
        let flag = getaddrinfo(host, zeroed(), &hints, &mut list);
        forget(hints);
        forget(host);

        // handle getaddrinfo error
        if flag != 0 {
            if !list.is_null() {
                freeaddrinfo(list);
            }
            return Err(build_error(flag));
        }

        // parse the responses
        collect_address(&mut addr, list)?;
        forget(list);
    }
    Ok(addr)
}

unsafe fn collect_address(vec: &mut Vec<IpAddr>, list: *mut addrinfo)
    -> Result<(),String>
{
    //sanity check
    if list.is_null() {
            return Err("null pointer error".to_string());
    }
    //alias a pointer
    let mut ptr: &mut addrinfo = {
        let x: usize = transmute(list);
        transmute(x)
    };
    let mut data = [0i8;100];
    // walk the linked list of responses
    loop {
        let tptr: *const sockaddr = transmute(ptr.ai_addr);
        let mutptr = data.as_mut_ptr();
        let flag = getnameinfo(tptr, ptr.ai_addrlen, mutptr, 100, zeroed(), 0, NI_NUMERICHOST);
        forget(mutptr);
        forget(tptr);
        if flag != 0 {
            forget(ptr);
            if !list.is_null() {
                freeaddrinfo(list);
            }
            return Err(build_error(flag));
        }
        match to_ip(&data) {
            Option::Some(ip) => vec.push(ip),
            _ => {}
        };

        // do we continue?
        if ptr.ai_next.is_null() {
            break;
        } else {
            ptr = transmute(ptr.ai_next);
        }
    }
    freeaddrinfo(list);
    forget(list);
    Ok(())
}

#[test]
fn test_dns_lookup() {
    match system_resolve_ipv4("google.com") {
        Ok(x) => {
            if x.len() == 0 {
                panic!("Found zero records for google.com");
            }
        }
        Err(e) => panic!("Error occured looking up google.com {:?}", e)
    };
}
