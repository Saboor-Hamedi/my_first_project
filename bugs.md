cargo run
   Compiling app v0.1.0 (B:\rust\my_first_project\app)
error[E0432]: unresolved import `crate::view_decision`                                                                  
  --> app\src\app.rs:17:12
   |
17 | use crate::view_decision::{render_decide_prompt, render_resolve_prompt};
   |            ^^^^^^^^^^^^^ could not find `view_decision` in the crate root

error[E0432]: unresolved import `crate::view_explain`                                                                   
  --> app\src\app.rs:19:12
   |
19 | use crate::view_explain::render_explain;
   |            ^^^^^^^^^^^^ could not find `view_explain` in the crate root

error[E0432]: unresolved import `crate::view_review`                                                                    
  --> app\src\app.rs:20:12
   |
20 | use crate::view_review::{render_recall_empty, render_review_answer, render_review_typing};
   |            ^^^^^^^^^^^ could not find `view_review` in the crate root

warning: unused import: `chrono::Local`                                                                                 
 --> app\src\commands.rs:9:5
  |
9 | use chrono::Local;
  |     ^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `crate::db_worker::DbMsg`
 --> app\src\input.rs:5:5
  |
5 | use crate::db_worker::DbMsg;
  |     ^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no variant or associated item named `ReviewTyping` found for enum `Mode` in the current scope             
   --> app\src\app.rs:336:19
    |
336 |             Mode::ReviewTyping => {
    |                   ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ReviewTyping` not found for this enum

error[E0599]: no variant or associated item named `ReviewAnswer` found for enum `Mode` in the current scope             
   --> app\src\app.rs:356:19
    |
356 |             Mode::ReviewAnswer => {
    |                   ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ReviewAnswer` not found for this enum

error[E0599]: no variant or associated item named `Explain` found for enum `Mode` in the current scope                  
   --> app\src\app.rs:369:19
    |
369 |             Mode::Explain => {
    |                   ^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `Explain` not found for this enum

error[E0599]: no variant or associated item named `DecidePrompt` found for enum `Mode` in the current scope             
   --> app\src\app.rs:385:19
    |
385 |             Mode::DecidePrompt => {
    |                   ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `DecidePrompt` not found for this enum

error[E0599]: no variant or associated item named `ResolvePrompt` found for enum `Mode` in the current scope            
   --> app\src\app.rs:401:19
    |
401 |             Mode::ResolvePrompt => {
    |                   ^^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ResolvePrompt` not found for this enum

error[E0061]: this function takes 9 arguments but 7 arguments were supplied                                             
   --> app\src\app.rs:412:17
    |
412 |                   render_stats(
    |                   ^^^^^^^^^^^^
...
415 | /                     self.total_cards,
416 | |                     &self.reviews_chart,
    | |                     ------------------- unexpected argument #4 of type `&std::vec::Vec<(String, usize)>`
417 | |                     &self.theme,
    | |_______________________________- three arguments of type `&DailyActivity`, `&[DailyActivity]`, and `(u64, u64, u64, usize)` are missing
418 |                       self.font_size,
    |                       -------------- expected `&str`, found `f32`
419 |                       lh,
    |                       -- expected `&str`, found `f32`
    |
note: function defined here
   --> app\src\view_stats.rs:44:8
    |
 44 | pub fn render_stats(
    |        ^^^^^^^^^^^^
...
 47 |     today_activity: &DailyActivity,
    |     ------------------------------
 48 |     history: &[DailyActivity],
    |     -------------------------
 49 |     lifetime: (u64, u64, u64, usize), // (total_seconds, total_keystrokes, total_words, active_days)
    |     --------------------------------
...
 52 |     today_date: &str,
    |     ----------------
 53 |     yesterday_date: &str,
    |     --------------------
help: did you mean
    |
412 ~                 render_stats(
413 +                     &painter,
414 +                     origin,
415 +                     /* &DailyActivity */,
416 +                     /* &[DailyActivity] */,
417 +                     /* (u64, u64, u64, usize) */,
418 +                     self.total_cards,
419 +                     &self.theme,
420 +                     /* &str */,
421 +                     /* &str */,
422 ~                 );
    |

error[E0599]: no variant or associated item named `ReviewTyping` found for enum `Mode` in the current scope             
   --> app\src\app.rs:446:23
    |
446 |                 Mode::ReviewTyping | Mode::ReviewAnswer => 1,
    |                       ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ReviewTyping` not found for this enum

error[E0599]: no variant or associated item named `ReviewAnswer` found for enum `Mode` in the current scope             
   --> app\src\app.rs:446:44
    |
446 |                 Mode::ReviewTyping | Mode::ReviewAnswer => 1,
    |                                            ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ReviewAnswer` not found for this enum

error[E0599]: no variant or associated item named `Explain` found for enum `Mode` in the current scope                  
   --> app\src\app.rs:447:23
    |
447 |                 Mode::Explain => 2,
    |                       ^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `Explain` not found for this enum

error[E0599]: no variant or associated item named `DecidePrompt` found for enum `Mode` in the current scope             
   --> app\src\app.rs:448:23
    |
448 |                 Mode::DecidePrompt | Mode::ResolvePrompt => 3,
    |                       ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `DecidePrompt` not found for this enum

error[E0599]: no variant or associated item named `ResolvePrompt` found for enum `Mode` in the current scope            
   --> app\src\app.rs:448:44
    |
448 |                 Mode::DecidePrompt | Mode::ResolvePrompt => 3,
    |                                            ^^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ResolvePrompt` not found for this enum

error[E0599]: no variant or associated item named `ReviewTyping` found for enum `Mode` in the current scope             
   --> app\src\app.rs:471:51
    |
471 | ...                   self.mode = Mode::ReviewTyping;
    |                                         ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `ReviewTyping` not found for this enum

error[E0599]: no variant or associated item named `Explain` found for enum `Mode` in the current scope                  
   --> app\src\app.rs:477:51
    |
477 | ...                   self.mode = Mode::Explain;
    |                                         ^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `Explain` not found for this enum

error[E0599]: no variant or associated item named `DecidePrompt` found for enum `Mode` in the current scope             
   --> app\src\app.rs:482:51
    |
482 | ...                   self.mode = Mode::DecidePrompt;
    |                                         ^^^^^^^^^^^^ variant or associated item not found in `Mode`
    |
   ::: app\src\mode.rs:4:1
    |
  4 | pub enum Mode {
    | ------------- variant or associated item `DecidePrompt` not found for this enum

error[E0507]: cannot move out of `*val` which is behind a shared reference                                              
   --> app\src\view_stats.rs:133:13
    |
133 |             *val,
    |             ^^^^ move occurs because `*val` has type `String`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
133 -             *val,
133 +             val.clone(),
    |

error[E0507]: cannot move out of `*sub` which is behind a shared reference
   --> app\src\view_stats.rs:142:13
    |
142 |             *sub,
    |             ^^^^ move occurs because `*sub` has type `String`, which does not implement the `Copy` trait
    |
help: consider cloning the value if the performance cost is acceptable
    |
142 -             *sub,
142 +             sub.clone(),
    |

Some errors have detailed explanations: E0061, E0432, E0507, E0599.                                                     
For more information about an error, try `rustc --explain E0061`.
warning: `app` (bin "app") generated 2 warnings                                                                         
error: could not compile `app` (bin "app") due to 19 previous errors; 2 warnings emitted
@Saboor ➜ my_first_project git(master)  
