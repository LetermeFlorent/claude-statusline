use super::Out;
use crate::messages;
use crate::render::text::{pct_text, reset_text};
use crate::render::Quota;

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
        let (palette, gradient) = match window {
            QuotaWindow::FiveHour | QuotaWindow::Spend => (self.cfg.pal_5h, self.cfg.grad_5h),
            QuotaWindow::SevenDay => (self.cfg.pal_7d, self.cfg.grad_7d),
        };
        let quota = quota.unwrap_or_default();
        self.label(window.label());
        self.line.push(' ');
        let start = match quota.pct {
            Some(p) if p >= self.cfg.quota_hot => palette.hot,
            Some(p) if p >= self.cfg.quota_warn => palette.warn,
            _ => palette.ok,
        };
        let last = self.gradient_bar(quota.pct, start, gradient);
        self.line.push(' ');
        self.value(&pct_text(quota.pct), quota.pct, last);
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
