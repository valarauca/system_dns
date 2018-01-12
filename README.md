System DNS
---

For valid `#[cfg(any(unix))]` systems this provides a simple API to
make DNS requests against the system resolver. This includes your
hosts file, and any configuration that maybe done to your system.

### To use this crate

```toml
#[dependencies]
system_dns = "1.0.0"
```

### Example

```rust
   extern crate system_dns;
   use system_dns::system_resolve_ipv4;
   let results: Vec<IpAddr> = match system_resolve_ipv4("google.com") {
   	Ok(x) => x,
        Err(e) => panic!("Failed to resolve `google.com`. Error {:?}", e)
   };
```
