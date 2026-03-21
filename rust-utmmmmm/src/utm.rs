// ════════════════════════════════════════════════════════════════════
// UTM core: types, constants, rule builder, encoding, infinite tape
// ════════════════════════════════════════════════════════════════════

// ── Direction ──
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
}

// ── UTM Symbol constants (indices into SYMBOL_NAMES) ──
pub const SYM_BLANK: u8 = 0; // "_"
pub const SYM_ZERO: u8 = 1; // "0"
pub const SYM_ONE: u8 = 2; // "1"
pub const SYM_X: u8 = 3; // "X"
pub const SYM_Y: u8 = 4; // "Y"
pub const SYM_HASH: u8 = 5; // "#"
pub const SYM_PIPE: u8 = 6; // "|"
pub const SYM_SEMI: u8 = 7; // ";"
pub const SYM_COMMA: u8 = 8; // ","
pub const SYM_CARET: u8 = 9; // "^"
pub const SYM_L: u8 = 10; // "l"
pub const SYM_D: u8 = 11; // "d"
pub const SYM_DOT: u8 = 12; // "."
pub const SYM_STAR: u8 = 13; // "*"
pub const SYM_GT: u8 = 14; // ">"
pub const SYM_DOLLAR: u8 = 15; // "$"
pub const N_SYMBOLS: usize = 16;

pub const SYMBOL_NAMES: [&str; N_SYMBOLS] = [
    "_", "0", "1", "X", "Y", "#", "|", ";", ",", "^", "l", "d", ".", "*", ">", "$",
];

// ── UTM State names (must match TypeScript allStates exactly) ──
pub const STATE_NAMES: &[&str] = &[
    "acc_final_home",    // 0
    "acc_rest_acc",      // 1
    "acc_rest_state",    // 2
    "accept",            // 3
    "accept_seek_home",  // 4
    "chk_acc_back2acc",  // 5
    "chk_acc_c0",        // 6
    "chk_acc_c0_find",   // 7
    "chk_acc_c1",        // 8
    "chk_acc_c1_find",   // 9
    "chk_acc_do_rest",   // 10
    "chk_acc_do_rest2",  // 11
    "chk_acc_fail_bit",  // 12
    "chk_acc_init",      // 13
    "chk_acc_into_acc",  // 14
    "chk_acc_next_entry", // 15
    "chk_acc_ok",        // 16
    "chk_acc_ok_acc",    // 17
    "chk_acc_ok_find",   // 18
    "chk_acc_ok_skip",   // 19
    "chk_acc_rest_state", // 20
    "mark_rule_no_match", // 21
    "ml_find_head",      // 22
    "ml_mark",           // 23
    "ml_nav",            // 24
    "ml_restore",        // 25
    "ml_s1",             // 26
    "ml_s2",             // 27
    "ml_s3",             // 28
    "move_left",         // 29
    "mr_ext_bc_next",    // 30
    "mr_ext_bc_ret",     // 31
    "mr_ext_bc0",        // 32
    "mr_ext_bc1",        // 33
    "mr_ext_h1",         // 34
    "mr_ext_h2",         // 35
    "mr_ext_h3",         // 36
    "mr_ext_home",       // 37
    "mr_ext_read_blank", // 38
    "mr_ext_rest_blank", // 39
    "mr_ext_to_blank",   // 40
    "mr_ext_write_head", // 41
    "mr_extend_init",    // 42
    "rej_final_home",    // 43
    "rej_rest_acc",      // 44
    "rej_rest_state",    // 45
    "reject",            // 46
    "reject_seek_home",  // 47
    "stf_skip_rest",     // 48
    "symf_skip_rest",    // 49
    "init_skip",         // 50
    "apply_read_nst",    // 51
    "cp_nsym_read",      // 52
    "init",              // 53
    "rd_read",           // 54
    "cp_nsym_c1_fb",     // 55
    "cp_nsym_done",      // 56
    "cp_nsym_nav2",      // 57
    "cp_nsym_rn_do",     // 58
    "cp_nsym_rn_s3",     // 59
    "mr_place_head",     // 60
    "mr_s3",             // 61
    "mr_skip_cell",      // 62
    "rd_sk2",            // 63
    "rd_sk4",            // 64
    "smc_rest_head",     // 65
    "smc_rest_sym",      // 66
    "smc_s3",            // 67
    "cp_nsym_c0_fb",     // 68
    "cp_nsym_c1_s3",     // 69
    "cmp_sym_read",      // 70
    "st_match_cleanup",  // 71
    "stm_go_left",       // 72
    "cp_nst_done",       // 73
    "cp_nst_rest_do",    // 74
    "cp_nst_rest_s1",    // 75
    "cp_nsym_nav",       // 76
    "cp_nsym_nav3",      // 77
    "cp_nsym_rn_s1",     // 78
    "cp_nsym_rn_s2",     // 79
    "mr_s1",             // 80
    "mr_s2",             // 81
    "rd_sk3",            // 82
    "rd_skip_to_dir",    // 83
    "smc_s1",            // 84
    "smc_s2",            // 85
    "smc_skip_st",       // 86
    "cp_nsym_c1_s1",     // 87
    "cp_nsym_c1_s2",     // 88
    "cmp_sym_c0_fb",     // 89
    "cp_nsym_c0_s3",     // 90
    "cmp_sym_fail",      // 91
    "cp_nsym_fn4",       // 92
    "cp_nst_c1_w",       // 93
    "cp_nst_c0_w",       // 94
    "cmp_sym_c1_fb",     // 95
    "cp_nsym_fn2",       // 96
    "cp_nsym_c0_s1",     // 97
    "cp_nsym_c0_s2",     // 98
    "move_right",        // 99
    "cmp_sym_nb2",       // 100
    "cp_nst_c1_s1",      // 101
    "cp_nsym_fn3",       // 102
    "cp_nsym_fnext",     // 103
    "cp_nst_c0_s1",      // 104
    "symf_rest_head",    // 105
    "symf_rest_sym",     // 106
    "cp_nst_next2",      // 107
    "cmp_sym_c0_s3",     // 108
    "cp_nst_next3",      // 109
    "cmp_sym_c1_s3",     // 110
    "symf_skip_st",      // 111
    "cp_nst_next",       // 112
    "stm_gs_sk1",        // 113
    "stm_restore_rule",  // 114
    "stm_restore_state", // 115
    "sym_skip_state",    // 116
    "cmp_sym_c0_s1",     // 117
    "cmp_sym_c0_s2",     // 118
    "cmp_sym_c1_s1",     // 119
    "cmp_sym_c1_s2",     // 120
    "cmp_sym_nextbit",   // 121
    "symf_deactivate",   // 122
    "cmp_st_read",       // 123
    "cmp_st_fail",       // 124
    "cmp_st_c0_find",    // 125
    "stf_go_prev",       // 126
    "stf_restore_rule",  // 127
    "stf_restore_state", // 128
    "cmp_st_c1_find",    // 129
    "cmp_st_nextbit",    // 130
    "cmp_st_c0_sk1",     // 131
    "cp_nsym_rn_fh",     // 132
    "mr_find_head",      // 133
    "smc_fh",            // 134
    "cp_nsym_c1_fh",     // 135
    "cmp_st_c1_sk1",     // 136
    "cp_nsym_rest_nav",  // 137
    "cp_nst_rest_nav",   // 138
    "sym_match_cleanup", // 139
    "mark_rule",         // 140
    "mr_nav",            // 141
    "cp_nsym_seek",      // 142
    "cp_nsym_c1",        // 143
    "cp_nsym_c0_fh",     // 144
    "read_dir",          // 145
    "smc_rest_done",     // 146
    "cp_nsym_c0",        // 147
    "cp_nst_c1",         // 148
    "cp_nst_c0",         // 149
    "cmp_sym_c0_fh",     // 150
    "cmp_sym_c1_fh",     // 151
    "init_seek_end",     // 152
    "cp_nsym_ret",       // 153
    "done_seek_home",    // 154
    "cp_nst_ret",        // 155
    "stm_goto_state",    // 156
    "stm_back_to_rule",  // 157
    "cmp_sym_c0",        // 158
    "cmp_sym_c1",        // 159
    "symf_seek_star",    // 160
    "cmp_sym_ok",        // 161
    "stf_find_star",     // 162
    "cmp_st_c0",         // 163
    "cmp_st_c1",         // 164
    "cmp_st_ok",         // 165
];

pub const N_UTM_STATES: usize = 166;

// ── TuringMachineSpec ──
pub struct TuringMachineSpec {
    pub n_states: usize,
    pub n_symbols: usize,
    pub initial: u8,
    pub accept: u8, // first accepting state (for UTM hot loop)
    pub blank: u8,
    pub accepting: Vec<bool>, // indexed by state
    pub transitions: Vec<Option<(u8, u8, Dir)>>, // 65536 entries, key = (state<<8)|sym
    pub state_names: Vec<&'static str>,
    pub symbol_names: Vec<&'static str>,
    pub ordered_rules: Vec<(u8, u8, u8, u8, Dir)>, // (st, sym, nst, nsym, dir) in map order
}

impl TuringMachineSpec {
    pub fn get_transition(&self, state: u8, sym: u8) -> Option<(u8, u8, Dir)> {
        self.transitions[((state as usize) << 8) | (sym as usize)]
    }

    pub fn is_accepting(&self, state: u8) -> bool {
        self.accepting.get(state as usize).copied().unwrap_or(false)
    }

    pub fn state_index(&self, name: &str) -> Option<u8> {
        self.state_names.iter().position(|&n| n == name).map(|i| i as u8)
    }

    pub fn symbol_index(&self, name: &str) -> Option<u8> {
        self.symbol_names.iter().position(|&n| n == name).map(|i| i as u8)
    }
}

// ── Helpers ──
pub fn num_bits(count: usize) -> usize {
    1.max((count.max(2) as f64).log2().ceil() as usize)
}

pub fn to_binary(index: usize, width: usize) -> Vec<u8> {
    let mut bits = Vec::with_capacity(width);
    for i in (0..width).rev() {
        bits.push(if (index >> i) & 1 == 1 { SYM_ONE } else { SYM_ZERO });
    }
    bits
}

fn from_binary_at(tape: &[u8], start: usize, width: usize) -> usize {
    let mut val = 0;
    for i in 0..width {
        let b = tape[start + i];
        val = val * 2 + match b {
            SYM_ONE | SYM_Y => 1,
            SYM_ZERO | SYM_X => 0,
            _ => panic!("invalid binary symbol at {}: {}", start + i, b),
        };
    }
    val
}

fn st(name: &str) -> u8 {
    STATE_NAMES
        .iter()
        .position(|&n| n == name)
        .unwrap_or_else(|| panic!("unknown UTM state: {}", name)) as u8
}

fn sym(name: &str) -> u8 {
    SYMBOL_NAMES
        .iter()
        .position(|&n| n == name)
        .unwrap_or_else(|| panic!("unknown UTM symbol: {}", name)) as u8
}

// ── RuleSet: transition table + ordered list for encoding ──
struct RuleSet {
    transitions: Vec<Option<(u8, u8, Dir)>>, // 65536 entries, key = (state<<8)|sym
    ordered: Vec<(u8, u8, u8, u8, Dir)>,     // (st, sym, nst, nsym, dir) in insertion order
}

impl RuleSet {
    fn new() -> Self {
        Self {
            transitions: vec![None; 65536],
            ordered: Vec::new(),
        }
    }

    fn add(&mut self, state: u8, s: u8, new_state: u8, new_sym: u8, dir: Dir) {
        let key = ((state as usize) << 8) | (s as usize);
        if self.transitions[key].is_some() {
            panic!(
                "Duplicate rule: state={} ({}), sym={} ({})",
                STATE_NAMES.get(state as usize).unwrap_or(&"?"),
                state,
                SYMBOL_NAMES.get(s as usize).unwrap_or(&"?"),
                s
            );
        }
        self.transitions[key] = Some((new_state, new_sym, dir));
        self.ordered.push((state, s, new_state, new_sym, dir));
    }

    /// Remove all rules for `state` from the transition table and ordered list.
    fn clear_state(&mut self, state: u8) {
        for &(st, s, _, _, _) in &self.ordered {
            if st == state {
                self.transitions[((st as usize) << 8) | (s as usize)] = None;
            }
        }
        self.ordered.retain(|&(st, _, _, _, _)| st != state);
    }
}

fn add_rule(m: &mut RuleSet, state: u8, s: u8, ns: u8, nsym: u8, dir: Dir) {
    m.add(state, s, ns, nsym, dir);
}

fn scan_right(m: &mut RuleSet, state: u8, syms: &[u8]) {
    for &s in syms {
        add_rule(m, state, s, state, s, Dir::Right);
    }
}

fn scan_left(m: &mut RuleSet, state: u8, syms: &[u8]) {
    for &s in syms {
        add_rule(m, state, s, state, s, Dir::Left);
    }
}

fn seek_home(m: &mut RuleSet, from: u8, to: u8) {
    scan_left(m, from, &[
        SYM_ZERO, SYM_ONE, SYM_X, SYM_Y, SYM_HASH, SYM_PIPE, SYM_SEMI, SYM_COMMA,
        SYM_CARET, SYM_DOT, SYM_STAR, SYM_GT, SYM_L, SYM_D,
    ]);
    add_rule(m, from, SYM_DOLLAR, to, SYM_DOLLAR, Dir::Right);
}

fn seek_star(m: &mut RuleSet, from: u8, to: u8) {
    scan_left(m, from, &[
        SYM_ZERO, SYM_ONE, SYM_X, SYM_Y, SYM_HASH, SYM_PIPE, SYM_SEMI, SYM_COMMA,
        SYM_CARET, SYM_DOT, SYM_L, SYM_D,
    ]);
    add_rule(m, from, SYM_STAR, to, SYM_STAR, Dir::Right);
}

// ════════════════════════════════════════════════════════════════════
// UTM Rule Builder
// ════════════════════════════════════════════════════════════════════

fn build_utm_rules() -> RuleSet {
    let mut r = RuleSet::new();

    // Symbol groups (matching TypeScript exactly)
    let rule_internals: &[u8] = &[SYM_ZERO, SYM_ONE, SYM_X, SYM_Y, SYM_PIPE, SYM_L, SYM_D];
    let rule_all: &[u8] = &[
        SYM_ZERO, SYM_ONE, SYM_X, SYM_Y, SYM_PIPE, SYM_L, SYM_D, SYM_SEMI, SYM_DOT, SYM_STAR,
    ];
    let bits: &[u8] = &[SYM_ZERO, SYM_ONE];
    let marked_bits: &[u8] = &[SYM_X, SYM_Y];
    let bits_and_marked: &[u8] = &[SYM_ZERO, SYM_ONE, SYM_X, SYM_Y];

    // ══════════════════════════════════════════════════════════════
    // PHASE 0: INIT
    // ══════════════════════════════════════════════════════════════
    add_rule(&mut r, st("init"), SYM_DOLLAR, st("init_skip"), SYM_DOLLAR, Dir::Right);
    add_rule(&mut r, st("init"), SYM_HASH, st("init_seek_end"), SYM_HASH, Dir::Right);
    add_rule(&mut r, st("init_skip"), SYM_HASH, st("init_seek_end"), SYM_HASH, Dir::Right);
    {
        let s = st("init_seek_end");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, s, &syms);
        add_rule(&mut r, s, SYM_HASH, st("mark_rule"), SYM_HASH, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 1: MARK RULE (right-to-left search)
    // ══════════════════════════════════════════════════════════════
    {
        let mr = st("mark_rule");
        scan_left(&mut r, mr, rule_internals);
        add_rule(&mut r, mr, SYM_SEMI, mr, SYM_SEMI, Dir::Left);
        add_rule(&mut r, mr, SYM_DOT, st("cmp_st_read"), SYM_STAR, Dir::Right);
        add_rule(&mut r, mr, SYM_HASH, st("mark_rule_no_match"), SYM_HASH, Dir::Right);
    }
    {
        let nm = st("mark_rule_no_match");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, nm, &syms);
        add_rule(&mut r, nm, SYM_HASH, st("chk_acc_init"), SYM_HASH, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 2: COMPARE STATE BITS
    // ══════════════════════════════════════════════════════════════
    add_rule(&mut r, st("cmp_st_read"), SYM_ZERO, st("cmp_st_c0"), SYM_X, Dir::Right);
    add_rule(&mut r, st("cmp_st_read"), SYM_ONE, st("cmp_st_c1"), SYM_Y, Dir::Right);
    add_rule(&mut r, st("cmp_st_read"), SYM_PIPE, st("st_match_cleanup"), SYM_PIPE, Dir::Right);

    for c in [0u8, 1u8] {
        let c_sym = if c == 0 { SYM_ZERO } else { SYM_ONE };
        let carry_name = if c == 0 { "cmp_st_c0" } else { "cmp_st_c1" };
        let sk1_name = if c == 0 { "cmp_st_c0_sk1" } else { "cmp_st_c1_sk1" };
        let find_name = if c == 0 { "cmp_st_c0_find" } else { "cmp_st_c1_find" };

        let carry = st(carry_name);
        scan_right(&mut r, carry, rule_all);
        add_rule(&mut r, carry, SYM_HASH, st(sk1_name), SYM_HASH, Dir::Right);

        let sk1 = st(sk1_name);
        let mut sk1_syms: Vec<u8> = bits_and_marked.to_vec();
        sk1_syms.push(SYM_SEMI);
        scan_right(&mut r, sk1, &sk1_syms);
        add_rule(&mut r, sk1, SYM_HASH, st(find_name), SYM_HASH, Dir::Right);

        let find = st(find_name);
        scan_right(&mut r, find, marked_bits);
        if c == 0 {
            add_rule(&mut r, find, SYM_ZERO, st("cmp_st_ok"), SYM_X, Dir::Left);
            add_rule(&mut r, find, SYM_ONE, st("cmp_st_fail"), SYM_Y, Dir::Left);
        } else {
            add_rule(&mut r, find, SYM_ONE, st("cmp_st_ok"), SYM_Y, Dir::Left);
            add_rule(&mut r, find, SYM_ZERO, st("cmp_st_fail"), SYM_X, Dir::Left);
        }
    }

    // Bit matched -> return to * to read next bit
    {
        seek_star(&mut r, st("cmp_st_ok"), st("cmp_st_nextbit"));
        let nb = st("cmp_st_nextbit");
        scan_right(&mut r, nb, marked_bits);
        add_rule(&mut r, nb, SYM_ZERO, st("cmp_st_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, nb, SYM_ONE, st("cmp_st_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, nb, SYM_PIPE, st("st_match_cleanup"), SYM_PIPE, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // STATE MATCH CLEANUP
    // ══════════════════════════════════════════════════════════════
    // First add wrong rules (to register state in insertion order), then clear & redo
    {
        let smc = st("st_match_cleanup");
        add_rule(&mut r, smc, SYM_ZERO, smc, SYM_ZERO, Dir::Right);
        add_rule(&mut r, smc, SYM_ONE, smc, SYM_ONE, Dir::Right);
    }
    r.clear_state(st("st_match_cleanup"));
    {
        let smc = st("st_match_cleanup");
        add_rule(&mut r, smc, SYM_ZERO, st("stm_go_left"), SYM_ZERO, Dir::Left);
        add_rule(&mut r, smc, SYM_ONE, st("stm_go_left"), SYM_ONE, Dir::Left);
        add_rule(&mut r, smc, SYM_PIPE, st("stm_go_left"), SYM_PIPE, Dir::Left);
    }
    {
        let gl = st("stm_go_left");
        add_rule(&mut r, gl, SYM_PIPE, st("stm_restore_rule"), SYM_PIPE, Dir::Left);
        scan_left(&mut r, gl, bits);
    }
    {
        let rr = st("stm_restore_rule");
        add_rule(&mut r, rr, SYM_X, rr, SYM_ZERO, Dir::Left);
        add_rule(&mut r, rr, SYM_Y, rr, SYM_ONE, Dir::Left);
        scan_left(&mut r, rr, bits);
        add_rule(&mut r, rr, SYM_STAR, st("stm_goto_state"), SYM_STAR, Dir::Right);
    }
    {
        let gs = st("stm_goto_state");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, gs, &syms);
        add_rule(&mut r, gs, SYM_HASH, st("stm_gs_sk1"), SYM_HASH, Dir::Right);
    }
    {
        let sk = st("stm_gs_sk1");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, sk, &syms);
        add_rule(&mut r, sk, SYM_HASH, st("stm_restore_state"), SYM_HASH, Dir::Right);
    }
    {
        let rs = st("stm_restore_state");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_HASH, st("stm_back_to_rule"), SYM_HASH, Dir::Left);
    }
    {
        seek_star(&mut r, st("stm_back_to_rule"), st("sym_skip_state"));
    }
    {
        let ss = st("sym_skip_state");
        scan_right(&mut r, ss, bits);
        add_rule(&mut r, ss, SYM_PIPE, st("cmp_sym_read"), SYM_PIPE, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // STATE MISMATCH
    // ══════════════════════════════════════════════════════════════
    {
        let sf = st("cmp_st_fail");
        scan_left(&mut r, sf, bits_and_marked);
        add_rule(&mut r, sf, SYM_HASH, st("stf_restore_state"), SYM_HASH, Dir::Right);
    }
    {
        let rs = st("stf_restore_state");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_HASH, st("stf_find_star"), SYM_HASH, Dir::Left);
    }
    {
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_HASH, SYM_PIPE, SYM_DOT, SYM_L, SYM_D]);
        scan_left(&mut r, st("stf_find_star"), &syms);
        add_rule(&mut r, st("stf_find_star"), SYM_STAR, st("stf_restore_rule"), SYM_DOT, Dir::Right);
    }
    {
        let rr = st("stf_restore_rule");
        add_rule(&mut r, rr, SYM_X, rr, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rr, SYM_Y, rr, SYM_ONE, Dir::Right);
        scan_right(&mut r, rr, bits);
        add_rule(&mut r, rr, SYM_PIPE, st("stf_go_prev"), SYM_PIPE, Dir::Left);
    }
    {
        let gp = st("stf_go_prev");
        scan_left(&mut r, gp, bits);
        add_rule(&mut r, gp, SYM_DOT, st("mark_rule"), SYM_DOT, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 3: COMPARE SYMBOL BITS
    // ══════════════════════════════════════════════════════════════
    add_rule(&mut r, st("cmp_sym_read"), SYM_ZERO, st("cmp_sym_c0"), SYM_X, Dir::Right);
    add_rule(&mut r, st("cmp_sym_read"), SYM_ONE, st("cmp_sym_c1"), SYM_Y, Dir::Right);
    add_rule(&mut r, st("cmp_sym_read"), SYM_PIPE, st("sym_match_cleanup"), SYM_PIPE, Dir::Right);

    for c in [0u8, 1u8] {
        let carry = st(if c == 0 { "cmp_sym_c0" } else { "cmp_sym_c1" });
        let s1 = st(if c == 0 { "cmp_sym_c0_s1" } else { "cmp_sym_c1_s1" });
        let s2 = st(if c == 0 { "cmp_sym_c0_s2" } else { "cmp_sym_c1_s2" });
        let s3 = st(if c == 0 { "cmp_sym_c0_s3" } else { "cmp_sym_c1_s3" });
        let fh = st(if c == 0 { "cmp_sym_c0_fh" } else { "cmp_sym_c1_fh" });
        let fb = st(if c == 0 { "cmp_sym_c0_fb" } else { "cmp_sym_c1_fb" });

        scan_right(&mut r, carry, rule_all);
        add_rule(&mut r, carry, SYM_HASH, s1, SYM_HASH, Dir::Right);

        // Skip ACC
        {
            let mut syms: Vec<u8> = bits_and_marked.to_vec();
            syms.push(SYM_SEMI);
            scan_right(&mut r, s1, &syms);
            add_rule(&mut r, s1, SYM_HASH, s2, SYM_HASH, Dir::Right);
        }
        // Skip STATE
        scan_right(&mut r, s2, bits_and_marked);
        add_rule(&mut r, s2, SYM_HASH, s3, SYM_HASH, Dir::Right);
        // Skip BLANK
        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, fh, SYM_HASH, Dir::Right);
        // Find ^
        {
            let mut syms: Vec<u8> = bits_and_marked.to_vec();
            syms.push(SYM_COMMA);
            scan_right(&mut r, fh, &syms);
            add_rule(&mut r, fh, SYM_CARET, fb, SYM_CARET, Dir::Right);
        }
        // Find next unmarked bit in head cell
        scan_right(&mut r, fb, marked_bits);
        if c == 0 {
            add_rule(&mut r, fb, SYM_ZERO, st("cmp_sym_ok"), SYM_X, Dir::Left);
            add_rule(&mut r, fb, SYM_ONE, st("cmp_sym_fail"), SYM_Y, Dir::Left);
        } else {
            add_rule(&mut r, fb, SYM_ONE, st("cmp_sym_ok"), SYM_Y, Dir::Left);
            add_rule(&mut r, fb, SYM_ZERO, st("cmp_sym_fail"), SYM_X, Dir::Left);
        }
    }

    // Symbol bit matched -> return to * to read next bit
    {
        seek_star(&mut r, st("cmp_sym_ok"), st("cmp_sym_nextbit"));
        let nb = st("cmp_sym_nextbit");
        scan_right(&mut r, nb, bits);
        add_rule(&mut r, nb, SYM_PIPE, st("cmp_sym_nb2"), SYM_PIPE, Dir::Right);
    }
    {
        let nb2 = st("cmp_sym_nb2");
        scan_right(&mut r, nb2, marked_bits);
        add_rule(&mut r, nb2, SYM_ZERO, st("cmp_sym_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, nb2, SYM_ONE, st("cmp_sym_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, nb2, SYM_PIPE, st("sym_match_cleanup"), SYM_PIPE, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MISMATCH
    // ══════════════════════════════════════════════════════════════
    {
        let sf = st("cmp_sym_fail");
        scan_left(&mut r, sf, bits_and_marked);
        add_rule(&mut r, sf, SYM_CARET, st("symf_rest_head"), SYM_CARET, Dir::Right);
    }
    {
        let rh = st("symf_rest_head");
        add_rule(&mut r, rh, SYM_X, rh, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rh, SYM_Y, rh, SYM_ONE, Dir::Right);
        scan_right(&mut r, rh, bits);
        add_rule(&mut r, rh, SYM_COMMA, st("symf_seek_star"), SYM_COMMA, Dir::Left);
        add_rule(&mut r, rh, SYM_BLANK, st("symf_seek_star"), SYM_BLANK, Dir::Left);
    }
    {
        seek_star(&mut r, st("symf_seek_star"), st("symf_skip_st"));
    }
    {
        let ss = st("symf_skip_st");
        scan_right(&mut r, ss, bits);
        add_rule(&mut r, ss, SYM_PIPE, st("symf_rest_sym"), SYM_PIPE, Dir::Right);
    }
    {
        let rs = st("symf_rest_sym");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_PIPE, st("symf_deactivate"), SYM_PIPE, Dir::Left);
    }
    {
        let da = st("symf_deactivate");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_PIPE);
        scan_left(&mut r, da, &syms);
        add_rule(&mut r, da, SYM_STAR, st("mark_rule"), SYM_DOT, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MATCH CLEANUP
    // ══════════════════════════════════════════════════════════════
    {
        let sc = st("sym_match_cleanup");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, sc, &syms);
        add_rule(&mut r, sc, SYM_HASH, st("smc_s1"), SYM_HASH, Dir::Right);
    }
    {
        let s1 = st("smc_s1");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, s1, &syms);
        add_rule(&mut r, s1, SYM_HASH, st("smc_s2"), SYM_HASH, Dir::Right);
    }
    {
        let s2 = st("smc_s2");
        scan_right(&mut r, s2, bits_and_marked);
        add_rule(&mut r, s2, SYM_HASH, st("smc_s3"), SYM_HASH, Dir::Right);
    }
    {
        let s3 = st("smc_s3");
        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, st("smc_fh"), SYM_HASH, Dir::Right);
    }
    {
        let fh = st("smc_fh");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_COMMA);
        scan_right(&mut r, fh, &syms);
        add_rule(&mut r, fh, SYM_CARET, st("smc_rest_head"), SYM_CARET, Dir::Right);
    }
    {
        let rh = st("smc_rest_head");
        add_rule(&mut r, rh, SYM_X, rh, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rh, SYM_Y, rh, SYM_ONE, Dir::Right);
        scan_right(&mut r, rh, bits);
        add_rule(&mut r, rh, SYM_COMMA, st("smc_rest_done"), SYM_COMMA, Dir::Left);
        add_rule(&mut r, rh, SYM_BLANK, st("smc_rest_done"), SYM_BLANK, Dir::Left);
    }
    {
        seek_star(&mut r, st("smc_rest_done"), st("smc_skip_st"));
    }
    {
        let ss = st("smc_skip_st");
        scan_right(&mut r, ss, bits);
        add_rule(&mut r, ss, SYM_PIPE, st("smc_rest_sym"), SYM_PIPE, Dir::Right);
    }
    {
        let rs = st("smc_rest_sym");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_PIPE, st("apply_read_nst"), SYM_PIPE, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 4: APPLY RULE - COPY NEW STATE
    // ══════════════════════════════════════════════════════════════
    add_rule(&mut r, st("apply_read_nst"), SYM_ZERO, st("cp_nst_c0"), SYM_X, Dir::Right);
    add_rule(&mut r, st("apply_read_nst"), SYM_ONE, st("cp_nst_c1"), SYM_Y, Dir::Right);
    add_rule(&mut r, st("apply_read_nst"), SYM_PIPE, st("cp_nst_done"), SYM_PIPE, Dir::Left);

    for c in [0u8, 1u8] {
        let carry = st(if c == 0 { "cp_nst_c0" } else { "cp_nst_c1" });
        let s1 = st(if c == 0 { "cp_nst_c0_s1" } else { "cp_nst_c1_s1" });
        let w = st(if c == 0 { "cp_nst_c0_w" } else { "cp_nst_c1_w" });
        let mark = if c == 0 { SYM_X } else { SYM_Y };

        scan_right(&mut r, carry, rule_all);
        add_rule(&mut r, carry, SYM_HASH, s1, SYM_HASH, Dir::Right);

        {
            let mut syms: Vec<u8> = bits_and_marked.to_vec();
            syms.push(SYM_SEMI);
            scan_right(&mut r, s1, &syms);
            add_rule(&mut r, s1, SYM_HASH, w, SYM_HASH, Dir::Right);
        }

        scan_right(&mut r, w, marked_bits);
        add_rule(&mut r, w, SYM_ZERO, st("cp_nst_ret"), mark, Dir::Left);
        add_rule(&mut r, w, SYM_ONE, st("cp_nst_ret"), mark, Dir::Left);
    }
    {
        seek_star(&mut r, st("cp_nst_ret"), st("cp_nst_next"));
    }
    {
        let n = st("cp_nst_next");
        scan_right(&mut r, n, bits);
        add_rule(&mut r, n, SYM_PIPE, st("cp_nst_next2"), SYM_PIPE, Dir::Right);
    }
    {
        let n2 = st("cp_nst_next2");
        scan_right(&mut r, n2, bits);
        add_rule(&mut r, n2, SYM_PIPE, st("cp_nst_next3"), SYM_PIPE, Dir::Right);
    }
    {
        let n3 = st("cp_nst_next3");
        scan_right(&mut r, n3, marked_bits);
        add_rule(&mut r, n3, SYM_ZERO, st("cp_nst_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, n3, SYM_ONE, st("cp_nst_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, n3, SYM_PIPE, st("cp_nst_done"), SYM_PIPE, Dir::Left);
    }

    // cp_nst_done: restore marks
    {
        let d = st("cp_nst_done");
        add_rule(&mut r, d, SYM_X, d, SYM_ZERO, Dir::Left);
        add_rule(&mut r, d, SYM_Y, d, SYM_ONE, Dir::Left);
        add_rule(&mut r, d, SYM_PIPE, st("cp_nst_rest_nav"), SYM_PIPE, Dir::Right);
    }
    {
        let nav = st("cp_nst_rest_nav");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, nav, &syms);
        add_rule(&mut r, nav, SYM_HASH, st("cp_nst_rest_s1"), SYM_HASH, Dir::Right);
    }
    {
        let s1 = st("cp_nst_rest_s1");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, s1, &syms);
        add_rule(&mut r, s1, SYM_HASH, st("cp_nst_rest_do"), SYM_HASH, Dir::Right);
    }
    {
        let rd = st("cp_nst_rest_do");
        add_rule(&mut r, rd, SYM_X, rd, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rd, SYM_Y, rd, SYM_ONE, Dir::Right);
        scan_right(&mut r, rd, bits);
        add_rule(&mut r, rd, SYM_HASH, st("cp_nsym_seek"), SYM_HASH, Dir::Left);
    }
    {
        seek_star(&mut r, st("cp_nsym_seek"), st("cp_nsym_nav"));
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 5: COPY NEW SYMBOL
    // ══════════════════════════════════════════════════════════════
    {
        let n = st("cp_nsym_nav");
        scan_right(&mut r, n, bits);
        add_rule(&mut r, n, SYM_PIPE, st("cp_nsym_nav2"), SYM_PIPE, Dir::Right);
    }
    {
        let n2 = st("cp_nsym_nav2");
        scan_right(&mut r, n2, bits);
        add_rule(&mut r, n2, SYM_PIPE, st("cp_nsym_nav3"), SYM_PIPE, Dir::Right);
    }
    {
        let n3 = st("cp_nsym_nav3");
        scan_right(&mut r, n3, bits);
        add_rule(&mut r, n3, SYM_PIPE, st("cp_nsym_read"), SYM_PIPE, Dir::Right);
    }

    add_rule(&mut r, st("cp_nsym_read"), SYM_ZERO, st("cp_nsym_c0"), SYM_X, Dir::Right);
    add_rule(&mut r, st("cp_nsym_read"), SYM_ONE, st("cp_nsym_c1"), SYM_Y, Dir::Right);
    add_rule(&mut r, st("cp_nsym_read"), SYM_PIPE, st("cp_nsym_done"), SYM_PIPE, Dir::Left);

    // Carry to head cell: skip rules, ACC, STATE, BLANK, find ^
    for c in [0u8, 1u8] {
        let carry = st(if c == 0 { "cp_nsym_c0" } else { "cp_nsym_c1" });
        let s1 = st(if c == 0 { "cp_nsym_c0_s1" } else { "cp_nsym_c1_s1" });
        let s2 = st(if c == 0 { "cp_nsym_c0_s2" } else { "cp_nsym_c1_s2" });
        let s3 = st(if c == 0 { "cp_nsym_c0_s3" } else { "cp_nsym_c1_s3" });
        let fh = st(if c == 0 { "cp_nsym_c0_fh" } else { "cp_nsym_c1_fh" });
        let fb = st(if c == 0 { "cp_nsym_c0_fb" } else { "cp_nsym_c1_fb" });
        let mark = if c == 0 { SYM_X } else { SYM_Y };

        scan_right(&mut r, carry, rule_all);
        add_rule(&mut r, carry, SYM_HASH, s1, SYM_HASH, Dir::Right);

        {
            let mut syms: Vec<u8> = bits_and_marked.to_vec();
            syms.push(SYM_SEMI);
            scan_right(&mut r, s1, &syms);
            add_rule(&mut r, s1, SYM_HASH, s2, SYM_HASH, Dir::Right);
        }
        scan_right(&mut r, s2, bits_and_marked);
        add_rule(&mut r, s2, SYM_HASH, s3, SYM_HASH, Dir::Right);

        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, fh, SYM_HASH, Dir::Right);

        {
            let mut syms: Vec<u8> = bits_and_marked.to_vec();
            syms.push(SYM_COMMA);
            scan_right(&mut r, fh, &syms);
            add_rule(&mut r, fh, SYM_CARET, fb, SYM_CARET, Dir::Right);
        }

        scan_right(&mut r, fb, marked_bits);
        add_rule(&mut r, fb, SYM_ZERO, st("cp_nsym_ret"), mark, Dir::Left);
        add_rule(&mut r, fb, SYM_ONE, st("cp_nsym_ret"), mark, Dir::Left);
    }
    {
        seek_star(&mut r, st("cp_nsym_ret"), st("cp_nsym_fnext"));
    }
    {
        let fn_ = st("cp_nsym_fnext");
        scan_right(&mut r, fn_, bits);
        add_rule(&mut r, fn_, SYM_PIPE, st("cp_nsym_fn2"), SYM_PIPE, Dir::Right);
    }
    {
        let fn2 = st("cp_nsym_fn2");
        scan_right(&mut r, fn2, bits);
        add_rule(&mut r, fn2, SYM_PIPE, st("cp_nsym_fn3"), SYM_PIPE, Dir::Right);
    }
    {
        let fn3 = st("cp_nsym_fn3");
        scan_right(&mut r, fn3, bits);
        add_rule(&mut r, fn3, SYM_PIPE, st("cp_nsym_fn4"), SYM_PIPE, Dir::Right);
    }
    {
        let fn4 = st("cp_nsym_fn4");
        scan_right(&mut r, fn4, marked_bits);
        add_rule(&mut r, fn4, SYM_ZERO, st("cp_nsym_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, fn4, SYM_ONE, st("cp_nsym_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, fn4, SYM_PIPE, st("cp_nsym_done"), SYM_PIPE, Dir::Left);
    }

    // cp_nsym_done: restore newsym field and head cell
    {
        let d = st("cp_nsym_done");
        add_rule(&mut r, d, SYM_X, d, SYM_ZERO, Dir::Left);
        add_rule(&mut r, d, SYM_Y, d, SYM_ONE, Dir::Left);
        add_rule(&mut r, d, SYM_PIPE, st("cp_nsym_rest_nav"), SYM_PIPE, Dir::Right);
    }
    {
        let nav = st("cp_nsym_rest_nav");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, nav, &syms);
        add_rule(&mut r, nav, SYM_HASH, st("cp_nsym_rn_s1"), SYM_HASH, Dir::Right);
    }
    {
        let s1 = st("cp_nsym_rn_s1");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, s1, &syms);
        add_rule(&mut r, s1, SYM_HASH, st("cp_nsym_rn_s2"), SYM_HASH, Dir::Right);
    }
    {
        let s2 = st("cp_nsym_rn_s2");
        scan_right(&mut r, s2, bits);
        add_rule(&mut r, s2, SYM_HASH, st("cp_nsym_rn_s3"), SYM_HASH, Dir::Right);
    }
    {
        let s3 = st("cp_nsym_rn_s3");
        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, st("cp_nsym_rn_fh"), SYM_HASH, Dir::Right);
    }
    {
        let fh = st("cp_nsym_rn_fh");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_COMMA);
        scan_right(&mut r, fh, &syms);
        add_rule(&mut r, fh, SYM_CARET, st("cp_nsym_rn_do"), SYM_CARET, Dir::Right);
    }
    {
        let rd = st("cp_nsym_rn_do");
        add_rule(&mut r, rd, SYM_X, rd, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rd, SYM_Y, rd, SYM_ONE, Dir::Right);
        scan_right(&mut r, rd, bits);
        add_rule(&mut r, rd, SYM_COMMA, st("read_dir"), SYM_COMMA, Dir::Left);
        add_rule(&mut r, rd, SYM_BLANK, st("read_dir"), SYM_BLANK, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 6: READ DIRECTION AND MOVE HEAD
    // ══════════════════════════════════════════════════════════════
    {
        seek_star(&mut r, st("read_dir"), st("rd_skip_to_dir"));
    }
    {
        let sk = st("rd_skip_to_dir");
        scan_right(&mut r, sk, bits);
        add_rule(&mut r, sk, SYM_PIPE, st("rd_sk2"), SYM_PIPE, Dir::Right);
    }
    {
        let sk2 = st("rd_sk2");
        scan_right(&mut r, sk2, bits);
        add_rule(&mut r, sk2, SYM_PIPE, st("rd_sk3"), SYM_PIPE, Dir::Right);
    }
    {
        let sk3 = st("rd_sk3");
        scan_right(&mut r, sk3, bits);
        add_rule(&mut r, sk3, SYM_PIPE, st("rd_sk4"), SYM_PIPE, Dir::Right);
    }
    {
        let sk4 = st("rd_sk4");
        scan_right(&mut r, sk4, bits);
        add_rule(&mut r, sk4, SYM_PIPE, st("rd_read"), SYM_PIPE, Dir::Right);
    }
    {
        add_rule(&mut r, st("rd_read"), SYM_L, st("move_left"), SYM_L, Dir::Left);
        add_rule(&mut r, st("rd_read"), SYM_D, st("move_right"), SYM_D, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE RIGHT
    // ══════════════════════════════════════════════════════════════
    {
        let mr = st("move_right");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.extend_from_slice(&[SYM_PIPE, SYM_L, SYM_D]);
        scan_left(&mut r, mr, &syms);
        add_rule(&mut r, mr, SYM_STAR, st("mr_nav"), SYM_DOT, Dir::Right);
    }
    {
        let nav = st("mr_nav");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, nav, &syms);
        add_rule(&mut r, nav, SYM_HASH, st("mr_s1"), SYM_HASH, Dir::Right);
    }
    {
        let s1 = st("mr_s1");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, s1, &syms);
        add_rule(&mut r, s1, SYM_HASH, st("mr_s2"), SYM_HASH, Dir::Right);
    }
    {
        let s2 = st("mr_s2");
        scan_right(&mut r, s2, bits);
        add_rule(&mut r, s2, SYM_HASH, st("mr_s3"), SYM_HASH, Dir::Right);
    }
    {
        let s3 = st("mr_s3");
        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, st("mr_find_head"), SYM_HASH, Dir::Right);
    }
    {
        let fh = st("mr_find_head");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_COMMA);
        scan_right(&mut r, fh, &syms);
        add_rule(&mut r, fh, SYM_CARET, st("mr_skip_cell"), SYM_GT, Dir::Right);
    }
    {
        let sc = st("mr_skip_cell");
        scan_right(&mut r, sc, bits);
        add_rule(&mut r, sc, SYM_COMMA, st("mr_place_head"), SYM_CARET, Dir::Left);
        add_rule(&mut r, sc, SYM_BLANK, st("mr_extend_init"), SYM_BLANK, Dir::Left);
    }
    {
        let ph = st("mr_place_head");
        scan_left(&mut r, ph, bits);
        add_rule(&mut r, ph, SYM_GT, st("done_seek_home"), SYM_COMMA, Dir::Left);
    }

    // EXTEND TAPE (move right past end)
    {
        let ei = st("mr_extend_init");
        scan_left(&mut r, ei, bits);
        add_rule(&mut r, ei, SYM_GT, st("mr_ext_to_blank"), SYM_COMMA, Dir::Right);
    }
    {
        let tb = st("mr_ext_to_blank");
        scan_right(&mut r, tb, bits);
        add_rule(&mut r, tb, SYM_BLANK, st("mr_ext_write_head"), SYM_CARET, Dir::Left);
    }
    {
        seek_home(&mut r, st("mr_ext_write_head"), st("mr_ext_home"));
    }
    {
        let eh = st("mr_ext_home");
        add_rule(&mut r, eh, SYM_HASH, st("mr_ext_h1"), SYM_HASH, Dir::Right);
    }
    {
        let h1 = st("mr_ext_h1");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, h1, &syms);
        add_rule(&mut r, h1, SYM_HASH, st("mr_ext_h2"), SYM_HASH, Dir::Right);
    }
    {
        let h2 = st("mr_ext_h2");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, h2, &syms);
        add_rule(&mut r, h2, SYM_HASH, st("mr_ext_h3"), SYM_HASH, Dir::Right);
    }
    {
        let h3 = st("mr_ext_h3");
        scan_right(&mut r, h3, bits);
        add_rule(&mut r, h3, SYM_HASH, st("mr_ext_read_blank"), SYM_HASH, Dir::Right);
    }
    {
        let rb = st("mr_ext_read_blank");
        add_rule(&mut r, rb, SYM_ZERO, st("mr_ext_bc0"), SYM_X, Dir::Right);
        add_rule(&mut r, rb, SYM_ONE, st("mr_ext_bc1"), SYM_Y, Dir::Right);
        add_rule(&mut r, rb, SYM_HASH, st("mr_ext_rest_blank"), SYM_HASH, Dir::Left);
    }
    for c in [0u8, 1u8] {
        let carry = st(if c == 0 { "mr_ext_bc0" } else { "mr_ext_bc1" });
        let c_sym = if c == 0 { SYM_ZERO } else { SYM_ONE };
        let mut syms: Vec<u8> = bits.to_vec();
        syms.extend_from_slice(&[SYM_HASH, SYM_COMMA, SYM_CARET]);
        scan_right(&mut r, carry, &syms);
        add_rule(&mut r, carry, SYM_BLANK, st("mr_ext_bc_ret"), c_sym, Dir::Left);
    }
    {
        let ret = st("mr_ext_bc_ret");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.extend_from_slice(&[SYM_HASH, SYM_COMMA, SYM_CARET]);
        scan_left(&mut r, ret, &syms);
        add_rule(&mut r, ret, SYM_X, st("mr_ext_bc_next"), SYM_X, Dir::Right);
        add_rule(&mut r, ret, SYM_Y, st("mr_ext_bc_next"), SYM_Y, Dir::Right);
    }
    {
        let next = st("mr_ext_bc_next");
        scan_right(&mut r, next, marked_bits);
        add_rule(&mut r, next, SYM_ZERO, st("mr_ext_bc0"), SYM_X, Dir::Right);
        add_rule(&mut r, next, SYM_ONE, st("mr_ext_bc1"), SYM_Y, Dir::Right);
        add_rule(&mut r, next, SYM_HASH, st("mr_ext_rest_blank"), SYM_HASH, Dir::Left);
    }
    {
        let rb = st("mr_ext_rest_blank");
        add_rule(&mut r, rb, SYM_X, rb, SYM_ZERO, Dir::Left);
        add_rule(&mut r, rb, SYM_Y, rb, SYM_ONE, Dir::Left);
        scan_left(&mut r, rb, bits);
        add_rule(&mut r, rb, SYM_HASH, st("done_seek_home"), SYM_HASH, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE LEFT
    // ══════════════════════════════════════════════════════════════
    {
        let ml = st("move_left");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.extend_from_slice(&[SYM_PIPE, SYM_L, SYM_D]);
        scan_left(&mut r, ml, &syms);
        add_rule(&mut r, ml, SYM_STAR, st("ml_nav"), SYM_DOT, Dir::Right);
    }
    {
        let nav = st("ml_nav");
        let mut syms: Vec<u8> = rule_internals.to_vec();
        syms.extend_from_slice(&[SYM_SEMI, SYM_DOT]);
        scan_right(&mut r, nav, &syms);
        add_rule(&mut r, nav, SYM_HASH, st("ml_s1"), SYM_HASH, Dir::Right);
    }
    {
        let s1 = st("ml_s1");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, s1, &syms);
        add_rule(&mut r, s1, SYM_HASH, st("ml_s2"), SYM_HASH, Dir::Right);
    }
    {
        let s2 = st("ml_s2");
        scan_right(&mut r, s2, bits);
        add_rule(&mut r, s2, SYM_HASH, st("ml_s3"), SYM_HASH, Dir::Right);
    }
    {
        let s3 = st("ml_s3");
        scan_right(&mut r, s3, bits);
        add_rule(&mut r, s3, SYM_HASH, st("ml_find_head"), SYM_HASH, Dir::Right);
    }
    {
        let fh = st("ml_find_head");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_COMMA);
        scan_right(&mut r, fh, &syms);
        add_rule(&mut r, fh, SYM_CARET, st("ml_mark"), SYM_GT, Dir::Left);
    }
    {
        let mk = st("ml_mark");
        scan_left(&mut r, mk, bits);
        add_rule(&mut r, mk, SYM_COMMA, st("ml_restore"), SYM_CARET, Dir::Right);
    }
    {
        let rs = st("ml_restore");
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_GT, st("done_seek_home"), SYM_COMMA, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 7: SEEK HOME AND RESTART
    // ══════════════════════════════════════════════════════════════
    seek_home(&mut r, st("done_seek_home"), st("init"));

    // ══════════════════════════════════════════════════════════════
    // PHASE 8: CHECK ACCEPT STATES
    // ══════════════════════════════════════════════════════════════
    {
        let ci = st("chk_acc_init");
        add_rule(&mut r, ci, SYM_HASH, st("rej_final_home"), SYM_HASH, Dir::Left);
        add_rule(&mut r, ci, SYM_ZERO, st("chk_acc_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, ci, SYM_ONE, st("chk_acc_c1"), SYM_Y, Dir::Right);
    }

    for c in [0u8, 1u8] {
        let carry = st(if c == 0 { "chk_acc_c0" } else { "chk_acc_c1" });
        let find = st(if c == 0 { "chk_acc_c0_find" } else { "chk_acc_c1_find" });

        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, carry, &syms);
        add_rule(&mut r, carry, SYM_HASH, find, SYM_HASH, Dir::Right);

        scan_right(&mut r, find, marked_bits);
        if c == 0 {
            add_rule(&mut r, find, SYM_ZERO, st("chk_acc_ok"), SYM_X, Dir::Left);
            add_rule(&mut r, find, SYM_ONE, st("chk_acc_fail_bit"), SYM_Y, Dir::Left);
        } else {
            add_rule(&mut r, find, SYM_ONE, st("chk_acc_ok"), SYM_Y, Dir::Left);
            add_rule(&mut r, find, SYM_ZERO, st("chk_acc_fail_bit"), SYM_X, Dir::Left);
        }
    }

    // Bit matched -> go back for next bit
    {
        let ok = st("chk_acc_ok");
        scan_left(&mut r, ok, bits_and_marked);
        add_rule(&mut r, ok, SYM_HASH, st("chk_acc_ok_acc"), SYM_HASH, Dir::Left);
    }
    {
        let oa = st("chk_acc_ok_acc");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_left(&mut r, oa, &syms);
        add_rule(&mut r, oa, SYM_HASH, st("chk_acc_ok_find"), SYM_HASH, Dir::Right);
    }
    {
        let of_ = st("chk_acc_ok_find");
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, of_, &syms);
        add_rule(&mut r, of_, SYM_X, st("chk_acc_ok_skip"), SYM_X, Dir::Right);
        add_rule(&mut r, of_, SYM_Y, st("chk_acc_ok_skip"), SYM_Y, Dir::Right);
        add_rule(&mut r, of_, SYM_HASH, st("accept_seek_home"), SYM_HASH, Dir::Left);
    }
    {
        let skip = st("chk_acc_ok_skip");
        scan_right(&mut r, skip, marked_bits);
        add_rule(&mut r, skip, SYM_ZERO, st("chk_acc_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, skip, SYM_ONE, st("chk_acc_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, skip, SYM_SEMI, st("accept_seek_home"), SYM_SEMI, Dir::Left);
        add_rule(&mut r, skip, SYM_HASH, st("accept_seek_home"), SYM_HASH, Dir::Left);
    }

    // Bit mismatch -> restore STATE marks, restore acc entry marks, try next entry
    {
        let fb = st("chk_acc_fail_bit");
        scan_left(&mut r, fb, bits_and_marked);
        add_rule(&mut r, fb, SYM_HASH, st("chk_acc_rest_state"), SYM_HASH, Dir::Right);
    }
    {
        let rs = st("chk_acc_rest_state");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_HASH, st("chk_acc_back2acc"), SYM_HASH, Dir::Left);
    }
    {
        let ba = st("chk_acc_back2acc");
        scan_left(&mut r, ba, bits);
        add_rule(&mut r, ba, SYM_HASH, st("chk_acc_into_acc"), SYM_HASH, Dir::Left);
    }
    {
        let ia = st("chk_acc_into_acc");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_left(&mut r, ia, &syms);
        add_rule(&mut r, ia, SYM_HASH, st("chk_acc_do_rest"), SYM_HASH, Dir::Right);
    }
    {
        let dr = st("chk_acc_do_rest");
        scan_right(&mut r, dr, bits);
        add_rule(&mut r, dr, SYM_X, st("chk_acc_do_rest2"), SYM_ZERO, Dir::Right);
        add_rule(&mut r, dr, SYM_Y, st("chk_acc_do_rest2"), SYM_ONE, Dir::Right);
        add_rule(&mut r, dr, SYM_SEMI, st("chk_acc_next_entry"), SYM_SEMI, Dir::Right);
        add_rule(&mut r, dr, SYM_HASH, st("reject_seek_home"), SYM_HASH, Dir::Left);
    }
    {
        let dr2 = st("chk_acc_do_rest2");
        add_rule(&mut r, dr2, SYM_X, dr2, SYM_ZERO, Dir::Right);
        add_rule(&mut r, dr2, SYM_Y, dr2, SYM_ONE, Dir::Right);
        scan_right(&mut r, dr2, bits);
        add_rule(&mut r, dr2, SYM_SEMI, st("chk_acc_next_entry"), SYM_SEMI, Dir::Right);
        add_rule(&mut r, dr2, SYM_HASH, st("reject_seek_home"), SYM_HASH, Dir::Left);
    }
    {
        let ne = st("chk_acc_next_entry");
        add_rule(&mut r, ne, SYM_ZERO, st("chk_acc_c0"), SYM_X, Dir::Right);
        add_rule(&mut r, ne, SYM_ONE, st("chk_acc_c1"), SYM_Y, Dir::Right);
        add_rule(&mut r, ne, SYM_HASH, st("reject_seek_home"), SYM_HASH, Dir::Left);
    }

    // Accept: restore ACCEPTSTATES and STATE, seek home
    {
        let ash = st("accept_seek_home");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_left(&mut r, ash, &syms);
        add_rule(&mut r, ash, SYM_HASH, st("acc_rest_acc"), SYM_HASH, Dir::Right);
    }
    {
        let ra = st("acc_rest_acc");
        add_rule(&mut r, ra, SYM_X, ra, SYM_ZERO, Dir::Right);
        add_rule(&mut r, ra, SYM_Y, ra, SYM_ONE, Dir::Right);
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, ra, &syms);
        add_rule(&mut r, ra, SYM_HASH, st("acc_rest_state"), SYM_HASH, Dir::Right);
    }
    {
        let rs = st("acc_rest_state");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_HASH, st("acc_final_home"), SYM_HASH, Dir::Left);
    }
    seek_home(&mut r, st("acc_final_home"), st("accept"));

    // Reject: restore marks
    {
        let rsh = st("reject_seek_home");
        let mut syms: Vec<u8> = bits_and_marked.to_vec();
        syms.push(SYM_SEMI);
        scan_left(&mut r, rsh, &syms);
        add_rule(&mut r, rsh, SYM_HASH, st("rej_rest_acc"), SYM_HASH, Dir::Right);
    }
    {
        let ra = st("rej_rest_acc");
        add_rule(&mut r, ra, SYM_X, ra, SYM_ZERO, Dir::Right);
        add_rule(&mut r, ra, SYM_Y, ra, SYM_ONE, Dir::Right);
        let mut syms: Vec<u8> = bits.to_vec();
        syms.push(SYM_SEMI);
        scan_right(&mut r, ra, &syms);
        add_rule(&mut r, ra, SYM_HASH, st("rej_rest_state"), SYM_HASH, Dir::Right);
    }
    {
        let rs = st("rej_rest_state");
        add_rule(&mut r, rs, SYM_X, rs, SYM_ZERO, Dir::Right);
        add_rule(&mut r, rs, SYM_Y, rs, SYM_ONE, Dir::Right);
        scan_right(&mut r, rs, bits);
        add_rule(&mut r, rs, SYM_HASH, st("rej_final_home"), SYM_HASH, Dir::Left);
    }
    seek_home(&mut r, st("rej_final_home"), st("reject"));

    r
}

// ════════════════════════════════════════════════════════════════════
// build_utm_spec: Assemble the full TuringMachineSpec for the UTM
// ════════════════════════════════════════════════════════════════════

pub fn build_utm_spec() -> TuringMachineSpec {
    let r = build_utm_rules();

    let mut accepting = vec![false; N_UTM_STATES];
    accepting[st("accept") as usize] = true;

    TuringMachineSpec {
        n_states: N_UTM_STATES,
        n_symbols: N_SYMBOLS,
        initial: st("init"),
        accept: st("accept"),
        blank: SYM_BLANK,
        accepting,
        transitions: r.transitions,
        state_names: STATE_NAMES.to_vec(),
        symbol_names: SYMBOL_NAMES.to_vec(),
        ordered_rules: r.ordered,
    }
}

// ════════════════════════════════════════════════════════════════════
// Encoding: encode an arbitrary TM + input into a UTM tape
// ════════════════════════════════════════════════════════════════════

/// Encode a guest TM spec into UTM tape symbols.
/// Layout: $ RULES # ACCEPTSTATES # STATE # BLANK # TAPE $
///
/// RULES: dot-separated entries, each = stateBits | symBits | newStateBits | newSymBits | dir
/// ACCEPTSTATES: semicolon-separated state encodings
/// STATE: current state bits
/// BLANK: blank symbol bits
/// TAPE: comma-separated cells, head cell prefixed with ^
pub fn encode_tape(
    guest: &TuringMachineSpec,
    input: &[u8],       // guest symbol indices
    head_pos: usize,    // usually 0
    initial_state: Option<u8>,
) -> Vec<u8> {
    let n_state_bits = num_bits(guest.n_states);
    let n_sym_bits = num_bits(guest.n_symbols);
    let init_state = initial_state.unwrap_or(guest.initial);

    let mut tape: Vec<u8> = Vec::new();
    tape.push(SYM_DOLLAR);

    // RULES section: # .rule1 ; .rule2 ; .rule3 ... #
    tape.push(SYM_HASH);
    let mut first_rule = true;
    for &(st_idx, sym_idx, nst_idx, nsym_idx, dir) in &guest.ordered_rules {
        if !first_rule {
            tape.push(SYM_SEMI);
        }
        first_rule = false;
        tape.push(SYM_DOT); // each rule prefixed with .
        tape.extend_from_slice(&to_binary(st_idx as usize, n_state_bits));
        tape.push(SYM_PIPE);
        tape.extend_from_slice(&to_binary(sym_idx as usize, n_sym_bits));
        tape.push(SYM_PIPE);
        tape.extend_from_slice(&to_binary(nst_idx as usize, n_state_bits));
        tape.push(SYM_PIPE);
        tape.extend_from_slice(&to_binary(nsym_idx as usize, n_sym_bits));
        tape.push(SYM_PIPE);
        tape.push(match dir {
            Dir::Left => SYM_L,
            Dir::Right => SYM_D,
        });
    }

    tape.push(SYM_HASH);

    // Encode accepting states
    let mut first_acc = true;
    for (i, &is_acc) in guest.accepting.iter().enumerate() {
        if is_acc {
            if !first_acc {
                tape.push(SYM_SEMI);
            }
            first_acc = false;
            tape.extend_from_slice(&to_binary(i, n_state_bits));
        }
    }

    tape.push(SYM_HASH);

    // Current state
    tape.extend_from_slice(&to_binary(init_state as usize, n_state_bits));

    tape.push(SYM_HASH);

    // Blank symbol encoding
    tape.extend_from_slice(&to_binary(guest.blank as usize, n_sym_bits));

    tape.push(SYM_HASH);

    // Tape cells
    let tape_len = if input.is_empty() { 1 } else { input.len() };
    for i in 0..tape_len {
        if i > 0 {
            tape.push(SYM_COMMA);
        }
        if i == head_pos {
            tape.push(SYM_CARET);
        }
        let sym_val = if i < input.len() { input[i] as usize } else { guest.blank as usize };
        tape.extend_from_slice(&to_binary(sym_val, n_sym_bits));
    }

    // No trailing $ — the tape extends infinitely to the right with blanks.
    // The UTM's InfiniteTape provides blanks beyond the encoded region.
    tape
}

// ════════════════════════════════════════════════════════════════════
// Decoding: extract guest state from the UTM tape
// ════════════════════════════════════════════════════════════════════

pub struct DecodedGuestState {
    pub state: usize,
    pub head_pos: usize,
    pub tape: Vec<usize>,  // guest symbol indices
}

/// Decode the UTM tape back into guest TM state.
pub fn decode_tape(utm_tape: &[u8], guest: &TuringMachineSpec) -> DecodedGuestState {
    let n_state_bits = num_bits(guest.n_states);
    let n_sym_bits = num_bits(guest.n_symbols);

    // Find the sections separated by #
    // Layout: $ #[0] RULES #[1] ACC #[2] STATE #[3] BLANK #[4] TAPE $
    let mut hashes: Vec<usize> = Vec::new();
    for (i, &s) in utm_tape.iter().enumerate() {
        if s == SYM_HASH {
            hashes.push(i);
        }
    }

    // STATE section starts after 3rd hash (index 2)
    let state_start = hashes[2] + 1;
    let state = from_binary_at(utm_tape, state_start, n_state_bits);

    // TAPE section starts after 5th hash (index 4)
    let tape_start = hashes[4] + 1;
    // Tape extends to the end of the slice (no trailing $)
    let tape_end = utm_tape.len();

    // Parse cells
    let tape_section = &utm_tape[tape_start..tape_end];
    let mut cells: Vec<usize> = Vec::new();
    let mut head_pos: usize = 0;
    let mut i = 0;
    let mut cell_idx = 0;
    while i < tape_section.len() {
        let s = tape_section[i];
        if s == SYM_BLANK || s == SYM_DOLLAR {
            break; // end of meaningful tape data
        }
        if s == SYM_COMMA {
            i += 1;
            cell_idx += 1;
            continue;
        }
        if s == SYM_CARET || s == SYM_GT {
            if s == SYM_CARET {
                head_pos = cell_idx;
            }
            i += 1;
            continue;
        }
        // Read n_sym_bits
        if i + n_sym_bits > tape_section.len() {
            break;
        }
        let val = from_binary_at(tape_section, i, n_sym_bits);
        cells.push(val);
        i += n_sym_bits;
    }

    DecodedGuestState {
        state,
        head_pos,
        tape: cells,
    }
}

// ════════════════════════════════════════════════════════════════════
// Infinite tape wrapper for running TMs
// ════════════════════════════════════════════════════════════════════

pub struct InfiniteTape {
    pub left: Vec<u8>,   // cells to the left of origin, left[0] = position -1
    pub right: Vec<u8>,  // cells from origin rightward, right[0] = position 0
    pub blank: u8,
}

impl InfiniteTape {
    pub fn new(initial: &[u8], blank: u8) -> Self {
        Self {
            left: Vec::new(),
            right: initial.to_vec(),
            blank,
        }
    }

    pub fn get(&self, pos: i64) -> u8 {
        if pos >= 0 {
            let idx = pos as usize;
            if idx < self.right.len() { self.right[idx] } else { self.blank }
        } else {
            let idx = (-pos - 1) as usize;
            if idx < self.left.len() { self.left[idx] } else { self.blank }
        }
    }

    pub fn set(&mut self, pos: i64, val: u8) {
        if pos >= 0 {
            let idx = pos as usize;
            while self.right.len() <= idx {
                self.right.push(self.blank);
            }
            self.right[idx] = val;
        } else {
            let idx = (-pos - 1) as usize;
            while self.left.len() <= idx {
                self.left.push(self.blank);
            }
            self.left[idx] = val;
        }
    }

    /// Extract a contiguous slice as Vec, from min_pos to max_pos inclusive.
    pub fn extract(&self, min_pos: i64, max_pos: i64) -> Vec<u8> {
        (min_pos..=max_pos).map(|p| self.get(p)).collect()
    }
}

// ════════════════════════════════════════════════════════════════════
// Run a TM (direct simulation, not via UTM)
// ════════════════════════════════════════════════════════════════════

pub enum RunResult {
    Accepted(i64),  // steps
    Rejected(i64),
    StepLimit(i64),
}

pub fn run_tm(
    spec: &TuringMachineSpec,
    tape: &mut InfiniteTape,
    head: &mut i64,
    state: &mut u8,
    max_steps: i64,
) -> RunResult {
    for step in 0..max_steps {
        let sym = tape.get(*head);
        match spec.get_transition(*state, sym) {
            None => {
                // No rule: check if accepting or rejecting
                if spec.is_accepting(*state) {
                    return RunResult::Accepted(step);
                } else {
                    return RunResult::Rejected(step);
                }
            }
            Some((ns, nsym, dir)) => {
                tape.set(*head, nsym);
                *state = ns;
                match dir {
                    Dir::Left => *head -= 1,
                    Dir::Right => *head += 1,
                }
            }
        }
    }
    RunResult::StepLimit(max_steps)
}

// ════════════════════════════════════════════════════════════════════
// Optimization hints for the UTM hot loop
// ════════════════════════════════════════════════════════════════════

/// States that are "scan right" (read symbol, write same, move right).
pub fn scan_right_states(spec: &TuringMachineSpec) -> Vec<bool> {
    let mut result = vec![false; spec.n_states];
    for s in 0..spec.n_states {
        let mut has_any = false;
        for sym in 0..spec.n_symbols {
            if let Some((ns, nsym, dir)) = spec.get_transition(s as u8, sym as u8) {
                if ns as usize == s && nsym == sym as u8 && matches!(dir, Dir::Right) {
                    has_any = true;
                }
            }
        }
        if has_any {
            result[s] = true;
        }
    }
    result
}

/// States that are "scan left" (read symbol, write same, move left).
pub fn scan_left_states(spec: &TuringMachineSpec) -> Vec<bool> {
    let mut result = vec![false; spec.n_states];
    for s in 0..spec.n_states {
        let mut has_any = false;
        for sym in 0..spec.n_symbols {
            if let Some((ns, nsym, dir)) = spec.get_transition(s as u8, sym as u8) {
                if ns as usize == s && nsym == sym as u8 && matches!(dir, Dir::Left) {
                    has_any = true;
                }
            }
        }
        if has_any {
            result[s] = true;
        }
    }
    result
}

// ════════════════════════════════════════════════════════════════════
// Tape formatting for debugging
// ════════════════════════════════════════════════════════════════════

pub fn format_tape(tape: &[u8]) -> String {
    tape.iter()
        .map(|&s| SYMBOL_NAMES.get(s as usize).unwrap_or(&"?").to_string())
        .collect::<Vec<_>>()
        .join("")
}

// ════════════════════════════════════════════════════════════════════
// Infinite UTM tape: encode a UTM simulating itself
// ════════════════════════════════════════════════════════════════════

/// Build the header portion of the infinite UTM tape (everything before the tape section).
/// This is the UTM encoding of the UTM itself with an empty initial tape.
/// Returns the header up to (but not including) the first `^`.
pub fn infinite_utm_tape_header() -> Vec<u8> {
    let utm = build_utm_spec();
    // Encode the UTM simulating itself with empty input
    let full_tape = encode_tape(&utm, &[], 0, None);
    // Find the position of the first ^ (start of tape section's head marker)
    let caret_pos = full_tape
        .iter()
        .position(|&s| s == SYM_CARET)
        .expect("encoded tape should contain ^");
    full_tape[..caret_pos].to_vec()
}

/// Compute the background symbol at global tape position `idx` of the infinite UTM tape.
///
/// The infinite UTM tape encodes "a UTM simulating itself on this very tape."
/// It is defined by a self-referential function:
///
///   background(idx) =
///     if idx < header.len():  header[idx]           -- the header is literal
///     else:
///       cell_idx = (idx - header.len()) / cell_size -- which tape cell are we in?
///       within   = (idx - header.len()) % cell_size -- where within that cell?
///       if within == 0:  ^ (for cell 0) or , (for others)  -- cell separator/head marker
///       else:  toBinary( background(cell_idx) , n_sym_bits )[within - 1]
///                         ^^^^^^^^^^^^^^^^^^^^
///                         RECURSIVE: the content of cell N is the binary encoding
///                         of whatever symbol sits at global position N of this tape.
///
/// This is well-founded because cell N encodes position N, and for N >= header.len(),
/// position N is inside cell (N - header.len()) / cell_size, which is always < N
/// (since cell_size >= 2). For N < header.len(), we hit the base case.
pub fn infinite_utm_tape_background(
    header: &[u8],
    n_sym_bits: usize,
    cell_size: usize,
    idx: usize,
) -> u8 {
    if idx < header.len() {
        return header[idx];
    }
    let offset = idx - header.len();
    let cell_idx = offset / cell_size;
    let within = offset % cell_size;
    if within == 0 {
        return if cell_idx == 0 { SYM_CARET } else { SYM_COMMA };
    }
    // Recurse: what symbol lives at global position `cell_idx`?
    let sym = infinite_utm_tape_background(header, n_sym_bits, cell_size, cell_idx);
    // Encode that symbol as binary, return the appropriate bit
    let bits = to_binary(sym as usize, n_sym_bits);
    bits[within - 1]
}
