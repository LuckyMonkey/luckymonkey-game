//! Deep links: the URL hash can open the game at a place, with something staged.
//!
//!   …/luckymonkey#node=data-lake                 skip the welcome gate, fly to a node, open its panel
//!   …/luckymonkey#node=mq-falls&zoom=1.6         …and zoom (0.5 - 2.5)
//!   …/luckymonkey#node=banana-sector&incident=ddos     start a shift with that incident on that node
//!   …/luckymonkey#node=auth-gate&forecast=cert         …or that forecast
//!   …/luckymonkey#shift                          skip the gate and start a shift
//!
//! The wiki documentation uses these for its "see it live" links, and the
//! screenshot tooling uses them to frame shots. Incident names are the fault
//! keys in tuning.json (crash, dns, dhcp, cable, route, ddos, worm, cert, leak,
//! disk, heat, power or lightning, deploy, migration, disparity, loop,
//! broadcast, vlan); forecast names are faults with a warning stage (worm,
//! cert, disk, deploy). Unknown values are ignored.

use bevy::prelude::*;

#[derive(Resource, Default, Clone, Debug)]
pub struct DeepLink {
    pub node: Option<String>,
    pub zoom: Option<f32>,
    pub incident: Option<String>,
    pub forecast: Option<String>,
    pub shift: bool,
}

impl DeepLink {
    /// Anything asked for at all: then the welcome gate is skipped.
    pub fn active(&self) -> bool {
        self.node.is_some() || self.zoom.is_some() || self.incident.is_some() || self.forecast.is_some() || self.shift
    }
}

/// `#a=b&c` → fields. Values are short ids; anything odd is dropped.
pub fn parse(hash: &str) -> DeepLink {
    let mut link = DeepLink::default();
    let clean = |v: &str| (v.len() <= 40 && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')).then(|| v.to_ascii_lowercase());
    for part in hash.trim_start_matches('#').split('&').filter(|p| !p.is_empty()) {
        let (k, v) = part.split_once('=').unwrap_or((part, ""));
        match k {
            "node" => link.node = clean(v),
            "zoom" => link.zoom = v.parse::<f32>().ok().filter(|z| z.is_finite()).map(|z| z.clamp(0.5, 2.5)),
            "incident" => link.incident = clean(v),
            "forecast" => link.forecast = clean(v),
            "shift" => link.shift = true,
            _ => {}
        }
    }
    link
}

fn location_hash() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window().and_then(|w| w.location().hash().ok()).unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}

pub fn plugin(app: &mut App) {
    app.insert_resource(parse(&location_hash()));
}
