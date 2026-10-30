use super::sha512_consts::{H0, K};

pub(crate) fn sha512(data: &[u8]) -> [u8; 64] {
    let mut h = H0;
    let full = data.len() - data.len() % 128;
    for block in data[..full].as_chunks::<128>().0 {
        compress(&mut h, block);
    }
    let rest = data.len() - full;
    let mut tail = [0u8; 256];
    tail[..rest].copy_from_slice(&data[full..]);
    tail[rest] = 0x80;
    let end = if rest < 112 { 128 } else { 256 };
    let bits = (data.len() as u128) * 8;
    tail[end - 16..end].copy_from_slice(&bits.to_be_bytes());
    for block in tail[..end].as_chunks::<128>().0 {
        compress(&mut h, block);
    }
    let mut out = [0u8; 64];
    for (chunk, word) in out.as_chunks_mut::<8>().0.iter_mut().zip(h) {
        *chunk = word.to_be_bytes();
    }
    out
}

fn compress(h: &mut [u64; 8], block: &[u8]) {
    let mut w = [0u64; 16];
    for (slot, word) in w.iter_mut().zip(block.as_chunks::<8>().0) {
        *slot = u64::from_be_bytes(*word);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for (t, k) in K.iter().enumerate() {
        if t >= 16 {
            let w15 = w[(t + 1) & 15];
            let w2 = w[(t + 14) & 15];
            let s0 = w15.rotate_right(1) ^ w15.rotate_right(8) ^ (w15 >> 7);
            let s1 = w2.rotate_right(19) ^ w2.rotate_right(61) ^ (w2 >> 6);
            w[t & 15] = w[t & 15]
                .wrapping_add(s0)
                .wrapping_add(w[(t + 9) & 15])
                .wrapping_add(s1);
        }
        let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
        let ch = (e & f) ^ (!e & g);
        let t1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(*k)
            .wrapping_add(w[t & 15]);
        let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
        let t2 = s0.wrapping_add((a & b) ^ (a & c) ^ (b & c));
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *x = x.wrapping_add(y);
    }
}
