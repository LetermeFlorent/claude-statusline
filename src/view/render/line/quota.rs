use super::Out;
use crate::messages;
use crate::render::text::{quota_text, reset_text, tenth};
use crate::render::Quota;

const LABEL_FIELD: usize = 3;
const QUOTA_FIELD: usize = 6;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum QuotaWindow {
    FiveHour,
    SevenDay,
    Spend,
}

impl QuotaWindow {
    fn label(self) -> &'static str {
        match self {
            QuotaWindow::FiveHour => "5h",
            QuotaWindow::SevenDay => messages::SEVEN_DAY_LABEL.text(),
            QuotaWindow::Spend => messages::SPEND_LABEL.text(),
        }
    }
}

impl Out<'_> {
    pub(super) fn quota(&mut self, window: QuotaWindow, quota: Option<Quota>, now: f64) {
        let palette = match window {
            QuotaWindow::FiveHour | QuotaWindow::Spend => self.cfg.pal_5h,
            QuotaWindow::SevenDay => self.cfg.pal_7d,
        };
        let quota = quota.unwrap_or_default();
        self.label(window.label());
        for _ in window.label().chars().count()..LABEL_FIELD {
            self.line.push(' ');
        }
        self.line.push(' ');
        let color = match quota.pct {
            Some(p) if tenth(p) >= self.cfg.quota_hot => palette.hot,
            Some(p) if tenth(p) >= self.cfg.quota_warn => palette.warn,
            _ => palette.ok,
        };
        let last = self.bar(quota.pct, color);
        self.line.push(' ');
        self.percent(&quota_text(quota.pct), QUOTA_FIELD, quota.pct, last);
        if !self.layout.resets {
            return;
        }
        if let Some(at) = quota.at {
            self.line.push(' ');
            self.label(&reset_text(at - now, window != QuotaWindow::FiveHour));
        } else if window == QuotaWindow::FiveHour {
            self.line.push(' ');
            self.label("--");
        }
    }
}
