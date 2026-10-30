use super::quota::QuotaWindow;
use super::{Out, RESET};
use crate::config::{Colors, Config};
use crate::messages;
use crate::render::screen::fit::Layout;
use crate::render::{Data, Depth};
use alloc::string::String;

const LINE_CAPACITY: usize = 2048;

fn has_windows(data: &Data) -> bool {
    data.quotas
        .is_some_and(|q| q.five_hour.is_some() || q.seven_day.is_some())
}

pub(in crate::view::render) fn draw(
    cfg: &Config,
    col: &Colors,
    data: &Data,
    now: f64,
    layout: Layout,
    depth: Depth,
) -> String {
    let mut out = Out {
        line: String::with_capacity(LINE_CAPACITY),
        cfg,
        col,
        layout,
        depth,
    };
    let mut first = true;
    if cfg.seg_ctx {
        out.sep(&mut first);
        out.ctx(data);
    }
    if let Some(quotas) = data.quotas {
        if cfg.seg_5h {
            out.sep(&mut first);
            out.quota(QuotaWindow::FiveHour, quotas.five_hour, now);
        }
        if cfg.seg_7d {
            out.sep(&mut first);
            out.quota(QuotaWindow::SevenDay, quotas.seven_day, now);
        }
    }
    if let Some(spend) = data.spend.filter(|_| cfg.seg_spend && !has_windows(data)) {
        out.sep(&mut first);
        out.quota(QuotaWindow::Spend, Some(spend), now);
    }
    if cfg.seg_model {
        out.sep(&mut first);
        out.model(data);
    }
    if !data.sys.is_empty() || data.updating {
        out.line.push('\n');
        first = true;
        for reading in &data.sys {
            out.sep(&mut first);
            out.system(reading);
        }
        if data.updating {
            out.sep(&mut first);
            out.label(messages::UPDATE_LABEL.text());
        }
    }
    out.line.push_str(RESET);
    out.line.push('\n');
    out.line
}
