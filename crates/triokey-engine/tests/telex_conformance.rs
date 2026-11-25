use triokey_engine::telex::compose;
use triokey_engine::ToneStyle::{self, New, Old};

fn run(keys: &str, style: ToneStyle) -> String {
    compose(&keys.chars().collect::<Vec<_>>(), style)
}

macro_rules! cases {
    ($($name:ident: $keys:literal, $style:ident => $want:literal;)*) => {
        $(#[test] fn $name() { assert_eq!(run($keys, $style), $want, "keys {:?} {:?}", $keys, $style); })*
    };
}

cases! {
    tieesng_old: "tieesng", Old => "tiếng";
    tieesng_new: "tieesng", New => "tiếng";
    tieengs_old: "tieengs", Old => "tiếng";
    tieengs_new: "tieengs", New => "tiếng";
    vieejt: "vieejt", Old => "việt";
    hoaf_old: "hoaf", Old => "hòa";
    hoaf_new: "hoaf", New => "hoà";
    thuys_old: "thuys", Old => "thúy";
    thuys_new: "thuys", New => "thuý";
    dduowngf: "dduowngf", Old => "đường";
    nguoiwf: "nguoiwf", Old => "người";
    uow: "uow", Old => "ươ";
    w_alone: "w", Old => "ư";
    quas: "quas", Old => "quá";
    gif: "gif", Old => "gì";
    ass: "ass", Old => "as";
    aaa: "aaa", Old => "aa";
    verr: "verr", Old => "ver";
    xooong: "xooong", Old => "xoong";
    usser: "usser", Old => "user";
    vieejt_cap: "Vieejt", Old => "Việt";
    vieejt_upper: "VIEEJT", Old => "VIỆT";
    dduowngf_cap: "DDuowngf", Old => "Đường";
    hoasz: "hoasz", Old => "hoa";
    khuyur: "khuyur", Old => "khuỷu";
    giaf: "giaf", Old => "già";
    quaanf: "quaanf", Old => "quần";
    truongwf: "truongwf", Old => "trường";
    truwowngf: "truwowngf", Old => "trường";
}
