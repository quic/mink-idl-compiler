// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

macro_rules! time {
    ($block: block, $note: literal) => {{
        let now = std::time::Instant::now();
        let ret = $block;
        let elapsed = now.elapsed();
        idlc_errors::info!("[{}] took {:?}", $note, elapsed);
        ret
    }};

    ($expr: expr, $note: literal) => {{
        $crate::timer::time!({ $expr }, $note)
    }};
}

pub(crate) use time;
