use utmmmmm::json_export::export_spec;
use utmmmmm::toy_machines::*;
use utmmmmm::utm;
use utmmmmm::utm::make_utm_spec;

fn main() {
    let utm_spec = make_utm_spec();

    let specs = vec![
        export_spec(
            &*ACCEPT_IMMEDIATELY_SPEC,
            "Accept Immediately",
            "Immediately accepts (initial state is accepting).",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                AccImmSymbol::Blank => '_',
            },
        ),
        export_spec(
            &*REJECT_IMMEDIATELY_SPEC,
            "Reject Immediately",
            "Immediately rejects (no transition, non-accepting).",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                RejImmSymbol::Blank => '_',
            },
        ),
        export_spec(
            &*FLIP_BITS_SPEC,
            "Flip Bits",
            "Flips 0s to 1s and vice versa, then halts at blank.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                FlipBitsSymbol::Blank => '_',
                FlipBitsSymbol::Zero => '0',
                FlipBitsSymbol::One => '1',
            },
        ),
        export_spec(
            &*CHECK_PALINDROME_SPEC,
            "Check Palindrome",
            "Checks if the input (over {A, B, C}) is a palindrome.",
            |s| format!("{:?}", s),
            |s| match s {
                CheckPalindromeState::Start => "start".to_string(),
                CheckPalindromeState::Accept => "accept".to_string(),
                CheckPalindromeState::SeekL => "returning to start".to_string(),
                CheckPalindromeState::SeekR(letter) => {
                    format!("saw {:?} at start, finding end", letter)
                }
                CheckPalindromeState::Check(letter) => {
                    format!("found the end; verifying last is {:?}", letter)
                }
            },
            |s| format!("{:?}", s),
            |s| match s {
                CheckPalindromeSymbol::Blank => '_',
                CheckPalindromeSymbol::Letter(Letter::A) => 'a',
                CheckPalindromeSymbol::Letter(Letter::B) => 'b',
                CheckPalindromeSymbol::Letter(Letter::C) => 'c',
                CheckPalindromeSymbol::Letter(Letter::D) => 'd',
                CheckPalindromeSymbol::Letter(Letter::E) => 'e',
                CheckPalindromeSymbol::Letter(Letter::F) => 'f',
                CheckPalindromeSymbol::Letter(Letter::G) => 'g',
                CheckPalindromeSymbol::Letter(Letter::H) => 'h',
                CheckPalindromeSymbol::Letter(Letter::I) => 'i',
                CheckPalindromeSymbol::Letter(Letter::J) => 'j',
                CheckPalindromeSymbol::Letter(Letter::K) => 'k',
                CheckPalindromeSymbol::Letter(Letter::L) => 'l',
                CheckPalindromeSymbol::Letter(Letter::M) => 'm',
                CheckPalindromeSymbol::Letter(Letter::N) => 'n',
                CheckPalindromeSymbol::Letter(Letter::O) => 'o',
                CheckPalindromeSymbol::Letter(Letter::P) => 'p',
                CheckPalindromeSymbol::Letter(Letter::Q) => 'q',
                CheckPalindromeSymbol::Letter(Letter::R) => 'r',
                CheckPalindromeSymbol::Letter(Letter::S) => 's',
                CheckPalindromeSymbol::Letter(Letter::T) => 't',
                CheckPalindromeSymbol::Letter(Letter::U) => 'u',
                CheckPalindromeSymbol::Letter(Letter::V) => 'v',
                CheckPalindromeSymbol::Letter(Letter::W) => 'w',
                CheckPalindromeSymbol::Letter(Letter::X) => 'x',
                CheckPalindromeSymbol::Letter(Letter::Y) => 'y',
                CheckPalindromeSymbol::Letter(Letter::Z) => 'z',
            },
        ),
        export_spec(
            &*DOUBLE_X_SPEC,
            "Double X",
            "Doubles a string of X's preceded by $. E.g. $XXX -> $XXXXXX.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                DoubleXSymbol::Blank => '_',
                DoubleXSymbol::Dollar => '$',
                DoubleXSymbol::X => 'X',
                DoubleXSymbol::Y => 'Y',
                DoubleXSymbol::Z => 'Z',
            },
        ),
        export_spec(
            &utm_spec,
            "Universal Turing Machine",
            "A universal Turing machine that can simulate any other TM given an encoded description on its tape.",
            |s| format!("{:?}", s),
            |s| match s {
                utm::State::Accept => "accepted!",
                utm::State::Reject => "rejected — no matching rule and state is not accepting",

                // Phase 0: Init
                utm::State::Init => "starting a new simulation cycle",
                utm::State::InitSkip => "skipping initial $ delimiter",
                utm::State::InitSeekEnd => "scanning right to find end of rules section",

                // Phase 1: Mark Rule
                utm::State::MarkRule => "searching left for next rule to try",
                utm::State::MarkRuleNoMatch => "no more rules to try; scanning right to check accept states",

                // Phase 2: Compare State
                utm::State::CmpStRead => "reading next state bit from current rule",
                utm::State::CmpStC0 => "carrying state bit 0 from rule; scanning right toward state section",
                utm::State::CmpStC0Find => "carrying state bit 0; finding next unmarked bit in state section",
                utm::State::CmpStC1 => "carrying state bit 1 from rule; scanning right toward state section",
                utm::State::CmpStC1Find => "carrying state bit 1; finding next unmarked bit in state section",
                utm::State::CmpStOk => "state bit matched; seeking back to rule's star marker",
                utm::State::CmpStNextbit => "at star marker; advancing to next state bit in rule",
                utm::State::CmpStFail => "state mismatch; scanning left to restore state section marks",

                // State Match Cleanup
                utm::State::StMatchCleanup => "state fully matched; beginning cleanup of marks",
                utm::State::StmGoLeft => "going left past state bits to reach pipe delimiter",
                utm::State::StmRestoreRule => "restoring marked state bits in rule back to 0/1",
                utm::State::StmGotoState => "scanning right past rules to reach state section",
                utm::State::StmRestoreState => "state match cleanup; restoring marked state section bits back to 0/1",
                utm::State::StmBackToRule => "state restored; seeking star marker to begin symbol comparison",

                // State Mismatch
                utm::State::StfRestoreState => "state mismatch; restoring state section marks to 0/1",
                utm::State::StfFindStar => "state mismatch; finding star marker to deactivate this rule",
                utm::State::StfRestoreRule => "state mismatch; restoring marked rule bits back to 0/1",
                utm::State::StfGoPrev => "state mismatch; going left to this rule's dot, then trying previous rule",
                utm::State::StfSkipRest => "state mismatch; skipping remaining rule fields",

                // Phase 3: Compare Symbol
                utm::State::SymSkipState => "state matched; skipping past state bits in rule to reach symbol field",
                utm::State::CmpSymRead => "reading next symbol bit from current rule",
                utm::State::CmpSymC0 => "carrying symbol bit 0 from rule; scanning right past rules",
                utm::State::CmpSymC0S1 => "carrying symbol bit 0; skipping past accept states",
                utm::State::CmpSymC0Fb => "carrying symbol bit 0; finding next unmarked bit in head cell",
                utm::State::CmpSymC1 => "carrying symbol bit 1 from rule; scanning right past rules",
                utm::State::CmpSymC1S1 => "carrying symbol bit 1; skipping past accept states",
                utm::State::CmpSymC1Fb => "carrying symbol bit 1; finding next unmarked bit in head cell",
                utm::State::CmpSymOk => "symbol bit matched; seeking back to rule's star marker",
                utm::State::CmpSymNextbit => "at star marker; advancing past state bits to next symbol bit",
                utm::State::CmpSymNb2 => "past state field pipe; finding next unmarked symbol bit in rule",
                utm::State::CmpSymFail => "symbol mismatch; scanning left to head marker to restore",

                // Symbol Mismatch
                utm::State::SymfRestHead => "symbol mismatch; restoring marked head cell bits back to 0/1",
                utm::State::SymfSeekStar => "symbol mismatch; finding star marker to deactivate this rule",
                utm::State::SymfSkipSt => "symbol mismatch; skipping state bits in rule to reach symbol field",
                utm::State::SymfRestSym => "symbol mismatch; restoring marked symbol bits in rule back to 0/1",
                utm::State::SymfDeactivate => "symbol mismatch; deactivating rule and trying previous one",
                utm::State::SymfSkipRest => "symbol mismatch; skipping remaining rule fields",

                // Symbol Match Cleanup
                utm::State::SymMatchCleanup => "rule fully matched! scanning right to restore marks before applying",
                utm::State::SmcS1 => "match cleanup; skipping past accept states section",
                utm::State::SmcRestHead => "match cleanup; restoring marked head cell bits back to 0/1",
                utm::State::SmcRestDone => "head cell restored; seeking star marker in rule",
                utm::State::SmcSkipSt => "match cleanup; skipping state bits in rule to reach symbol field",
                utm::State::SmcRestSym => "match cleanup; restoring marked symbol bits in rule back to 0/1",

                // Phase 4: Copy New State
                utm::State::ApplyReadNst => "reading next new-state bit from matched rule",
                utm::State::CpNstC0 => "carrying new-state bit 0; scanning right toward state section",
                utm::State::CpNstC0W => "carrying new-state bit 0; writing it into state section",
                utm::State::CpNstC1 => "carrying new-state bit 1; scanning right toward state section",
                utm::State::CpNstC1W => "carrying new-state bit 1; writing it into state section",
                utm::State::CpNstRet => "new-state bit written; seeking back to star marker for next bit",
                utm::State::CpNstNext => "copy new state; at star, skipping past state bits in rule",
                utm::State::CpNstNext2 => "copy new state; skipping past symbol bits in rule",
                utm::State::CpNstNext3 => "finding next unmarked new-state bit to copy",
                utm::State::CpNstDone => "all new-state bits copied; restoring marks in new-state field",
                utm::State::CpNstRestNav => "copy new state; navigating right to restore accept/state marks",
                utm::State::CpNstRestDo => "copy new state; restoring remaining marks in state section",

                // Phase 5: Copy New Symbol
                utm::State::CpNsymSeek => "new state written; seeking star marker to begin copying new symbol",
                utm::State::CpNsymNav => "copy new symbol; at star, skipping past state bits in rule",
                utm::State::CpNsymNav2 => "copy new symbol; skipping past symbol bits in rule",
                utm::State::CpNsymNav3 => "copy new symbol; skipping past new-state bits to reach new-symbol field",
                utm::State::CpNsymRead => "reading next new-symbol bit from matched rule",
                utm::State::CpNsymC0 => "carrying new-symbol bit 0; scanning right past rules",
                utm::State::CpNsymC0S1 => "carrying new-symbol bit 0; skipping past accept states",
                utm::State::CpNsymC0S2 => "carrying new-symbol bit 0; skipping past state section",
                utm::State::CpNsymC0S3 => "carrying new-symbol bit 0; skipping past blank section",
                utm::State::CpNsymC0Fh => "carrying new-symbol bit 0; finding head marker in tape",
                utm::State::CpNsymC0Fb => "carrying new-symbol bit 0; finding next unmarked bit in head cell",
                utm::State::CpNsymC1 => "carrying new-symbol bit 1; scanning right past rules",
                utm::State::CpNsymC1S1 => "carrying new-symbol bit 1; skipping past accept states",
                utm::State::CpNsymC1S2 => "carrying new-symbol bit 1; skipping past state section",
                utm::State::CpNsymC1S3 => "carrying new-symbol bit 1; skipping past blank section",
                utm::State::CpNsymC1Fh => "carrying new-symbol bit 1; finding head marker in tape",
                utm::State::CpNsymC1Fb => "carrying new-symbol bit 1; finding next unmarked bit in head cell",
                utm::State::CpNsymRet => "new-symbol bit written; seeking back to star marker for next bit",
                utm::State::CpNsymFnext => "copy new symbol return; at star, skipping past state bits in rule",
                utm::State::CpNsymFn2 => "copy new symbol return; skipping past symbol bits in rule",
                utm::State::CpNsymFn3 => "copy new symbol return; skipping past new-state bits in rule",
                utm::State::CpNsymFn4 => "finding next unmarked new-symbol bit to copy",
                utm::State::CpNsymDone => "all new-symbol bits copied; restoring marks in new-symbol field",
                utm::State::CpNsymRestNav => "copy new symbol; navigating right to restore tape section marks",
                utm::State::CpNsymRnS1 => "copy new symbol restore; skipping past accept states",
                utm::State::CpNsymRnS2 => "copy new symbol restore; skipping past state section",
                utm::State::CpNsymRnS3 => "copy new symbol restore; skipping past blank section",
                utm::State::CpNsymRnFh => "copy new symbol restore; finding head marker in tape",
                utm::State::CpNsymRnDo => "restoring marked head cell bits back to 0/1",

                // Phase 6: Read Direction
                utm::State::ReadDir => "new symbol written; seeking star marker to read direction",
                utm::State::RdSkipToDir => "read direction; skipping past state bits in rule",
                utm::State::RdSk2 => "read direction; skipping past symbol bits in rule",
                utm::State::RdSk3 => "read direction; skipping past new-state bits in rule",
                utm::State::RdSk4 => "read direction; skipping past new-symbol bits in rule",
                utm::State::RdRead => "reading direction: L or R",

                // Move Right
                utm::State::MoveRight => "direction is R; scanning left to deactivate star marker",
                utm::State::MrNav => "move right; scanning right past rules toward tape",
                utm::State::MrS1 => "move right; skipping past accept states",
                utm::State::MrS2 => "move right; skipping past state section",
                utm::State::MrS3 => "move right; skipping past blank section",
                utm::State::MrFindHead => "move right; finding head marker in tape",
                utm::State::MrSkipCell => "move right; skipping current cell to reach next cell",
                utm::State::MrPlaceHead => "move right; placing head marker at new position",

                // Move Right — Extend Tape
                utm::State::MrExtendInit => "move right; hit end of tape, preparing to extend",
                utm::State::MrExtToBlank => "extending tape; scanning right to place head at new blank cell",
                utm::State::MrExtWriteHead => "extending tape; head placed, seeking home to copy blank symbol",
                utm::State::MrExtHome => "extending tape; at home, navigating to blank symbol definition",
                utm::State::MrExtH1 => "extending tape; skipping past rules section",
                utm::State::MrExtH2 => "extending tape; skipping past accept states",
                utm::State::MrExtH3 => "extending tape; skipping past state section to reach blank definition",
                utm::State::MrExtReadBlank => "extending tape; reading next blank symbol bit to copy",
                utm::State::MrExtBc0 => "extending tape; carrying blank bit 0 to new cell",
                utm::State::MrExtBc1 => "extending tape; carrying blank bit 1 to new cell",
                utm::State::MrExtBcRet => "extending tape; blank bit written, returning for next bit",
                utm::State::MrExtBcNext => "extending tape; finding next unmarked blank bit to copy",
                utm::State::MrExtRestBlank => "extending tape; restoring blank section marks back to 0/1",

                // Move Left
                utm::State::MoveLeft => "direction is L; scanning left to deactivate star marker",
                utm::State::MlNav => "move left; scanning right past rules toward tape",
                utm::State::MlS1 => "move left; skipping past accept states",
                utm::State::MlS2 => "move left; skipping past state section",
                utm::State::MlS3 => "move left; skipping past blank section",
                utm::State::MlFindHead => "move left; finding head marker in tape",
                utm::State::MlMark => "move left; scanning left past previous cell to place head",
                utm::State::MlRestore => "move left; restoring old head position as comma separator",

                // Phase 7: Seek Home
                utm::State::DoneSeekHome => "head moved; seeking $ to restart simulation cycle",

                // Phase 8: Check Accept States
                utm::State::ChkAccInit => "no rule matched; beginning to check if current state is accepting",
                utm::State::ChkAccC0 => "checking accept; carrying bit 0 from accept entry toward state section",
                utm::State::ChkAccC0Find => "checking accept; carrying bit 0, finding next unmarked state bit",
                utm::State::ChkAccC1 => "checking accept; carrying bit 1 from accept entry toward state section",
                utm::State::ChkAccC1Find => "checking accept; carrying bit 1, finding next unmarked state bit",
                utm::State::ChkAccOk => "accept check bit matched; going back for next bit",
                utm::State::ChkAccOkAcc => "accept check bit matched; scanning left through accept section",
                utm::State::ChkAccOkFind => "accept check; finding next marked bit in accept entry",
                utm::State::ChkAccOkSkip => "accept check; skipping past already-matched bits",
                utm::State::ChkAccFailBit => "accept check bit mismatch; scanning left to restore marks",
                utm::State::ChkAccRestState => "accept check mismatch; restoring state section marks to 0/1",
                utm::State::ChkAccBack2acc => "accept check mismatch; returning to accept states section",
                utm::State::ChkAccIntoAcc => "accept check mismatch; finding current accept entry to restore",
                utm::State::ChkAccDoRest => "accept check mismatch; restoring marks in current accept entry",
                utm::State::ChkAccDoRest2 => "accept check mismatch; continuing to restore accept entry marks",
                utm::State::ChkAccNextEntry => "accept check; moving to next accept state entry to compare",

                // Accept
                utm::State::AcceptSeekHome => "state is accepting! restoring marks and seeking home",
                utm::State::AccRestAcc => "accepting; restoring accept states section marks to 0/1",
                utm::State::AccRestState => "accepting; restoring state section marks to 0/1",
                utm::State::AccFinalHome => "accepting; seeking $ to enter final accept state",

                // Reject
                utm::State::RejectSeekHome => "state is not accepting; restoring marks and seeking home",
                utm::State::RejRestAcc => "rejecting; restoring accept states section marks to 0/1",
                utm::State::RejRestState => "rejecting; restoring state section marks to 0/1",
                utm::State::RejFinalHome => "rejecting; seeking $ to enter final reject state",

                // Load (SYMCACHE)
                utm::State::LdC0 | utm::State::LdC1 => "load: carrying bit from tape to SYMCACHE",
                utm::State::LdC0Tp | utm::State::LdC1Tp => "load: scanning left through tape",
                utm::State::LdC0Bl | utm::State::LdC1Bl => "load: scanning left through blank section",
                utm::State::LdC0Sc | utm::State::LdC1Sc => "load: scanning left through SYMCACHE",
                utm::State::LdC0Wr | utm::State::LdC1Wr => "load: writing bit in SYMCACHE",
                utm::State::LdRead => "load: reading next bit from tape head cell",
                utm::State::LdRetSc => "load: returning right through SYMCACHE",
                utm::State::LdRetBl => "load: returning right through blank section",
                utm::State::LdRetFh => "load: returning right through tape to find head",
                utm::State::LdDone => "load: all bits loaded, starting restore",
                utm::State::LdRestTp => "load: restoring tape head cell marks",
                utm::State::LdNavSc1 => "load: navigating left to SYMCACHE through tape",
                utm::State::LdNavSc2 => "load: navigating left through blank section",
                utm::State::LdNavSc3 => "load: navigating left through SYMCACHE",
                utm::State::LdRestSc => "load: restoring SYMCACHE marks",
                utm::State::LdSeekHead => "load: seeking head marker after move right",
                utm::State::MlNavHead => "load: seeking head marker after move left",

                // Noop compact rule handling
                utm::State::NpNextbit => "noop rule; at caret marker, skipping marked bits to read next symbol bit",
                utm::State::NpMatchPre => "noop rule matched; scanning left to caret to restore before cleanup",
                utm::State::NpSmcHandler => "noop rule; restoring marks and skipping to direction after match",
                utm::State::NpReadDir => "noop rule; reading direction L/R",
                utm::State::NpSymfRestore => "noop rule mismatch; restoring current alternative, trying next",
            }.to_string(),
            |s| format!("{:?}", s),
            |s| match s {
                utm::Symbol::Blank => '_',
                utm::Symbol::Zero => '0',
                utm::Symbol::One => '1',
                utm::Symbol::X => 'X',
                utm::Symbol::Y => 'Y',
                utm::Symbol::Hash => '#',
                utm::Symbol::Pipe => '|',
                utm::Symbol::Semi => ';',
                utm::Symbol::Comma => ',',
                utm::Symbol::Caret => '^',
                utm::Symbol::L => 'L',
                utm::Symbol::R => 'R',
                utm::Symbol::Dot => '.',
                utm::Symbol::Star => '*',
                utm::Symbol::Gt => '>',
                utm::Symbol::Dollar => '$',
            },
        ),
    ];

    println!("{}", serde_json::to_string_pretty(&specs).unwrap());
}
