use actix_web::guard;
use std::net::IpAddr;

/// 内部アクセスかどうかを判定するカスタムガード
pub fn is_internal(ctx: &guard::GuardContext<'_>) -> bool {
	if let Some(addr) = ctx.head().peer_addr {
		let ip = addr.ip();

		// ループバック(127.0.0.1) または プライベートネットワーク(10.x, 172.16.x, 192.168.x)なら許可
		ip.is_loopback()
			|| match ip {
				IpAddr::V4(ipv4) => ipv4.is_private(),
				_ => false,
			}
	} else {
		false
	}
}
