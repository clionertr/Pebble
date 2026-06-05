const KNOWN_TRACKERS: &[&str] = &[
    "mailchimp.com",
    "list-manage.com",
    "hubspot.com",
    "sendgrid.net",
    "mailgun.org",
    "constantcontact.com",
    "campaign-archive.com",
    "exacttarget.com",
    "sailthru.com",
    "returnpath.net",
    "litmus.com",
    "bananatag.com",
    "yesware.com",
    "mailtrack.io",
    "getnotify.com",
    "streak.com",
    "cirrusinsight.com",
    "boomeranggmail.com",
    "mixmax.com",
    "superhuman.com",
    "facebook.com",
    "google-analytics.com",
    "doubleclick.net",
    "pixel.wp.com",
    "open.convertkit.com",
    "cmail19.com",
    "cmail20.com",
    "createsend.com",
    "intercom.io",
    "drip.com",
    "mandrillapp.com",
];

pub fn is_known_tracker(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    KNOWN_TRACKERS.iter().any(|t| domain_lower.contains(t))
}

pub fn is_tracking_pixel(width: Option<&str>, height: Option<&str>) -> bool {
    /// 将属性值解析为像素数；无法解析的（如 "100%"）返回 None，不参与跟踪像素判定。
    fn parse_pixel_dim(val: &str) -> Option<u32> {
        val.parse::<u32>().ok()
    }

    match (
        width.and_then(parse_pixel_dim),
        height.and_then(parse_pixel_dim),
    ) {
        // Both dimensions explicitly set to <= 1 — classic tracking pixel
        (Some(w), Some(h)) => w <= 1 && h <= 1,
        // One dimension present and <= 1, other absent — likely a pixel
        (Some(v), None) | (None, Some(v)) => v <= 1,
        // Both absent, or values are non-numeric (e.g. "100%") — cannot determine
        // from dimensions alone; don't flag here.
        (None, None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_tracker_domains() {
        assert!(is_known_tracker("tracking.mailchimp.com"));
        assert!(is_known_tracker("t.hubspot.com"));
        assert!(is_known_tracker("email.sendgrid.net"));
        assert!(!is_known_tracker("example.com"));
        assert!(!is_known_tracker("google.com"));
    }

    #[test]
    fn test_tracking_pixel_detection() {
        assert!(is_tracking_pixel(Some("1"), Some("1")));
        assert!(is_tracking_pixel(Some("0"), Some("0")));
        assert!(!is_tracking_pixel(Some("100"), Some("50")));
        assert!(!is_tracking_pixel(None, None)); // missing both dimensions — inconclusive, checked by other heuristics
        assert!(is_tracking_pixel(Some("1"), None)); // one dimension <= 1, other absent
        assert!(is_tracking_pixel(None, Some("0"))); // one dimension <= 1, other absent
    }
}
