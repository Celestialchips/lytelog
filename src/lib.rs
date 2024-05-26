use std::{
    io::{self, Write},
    sync::{atomic::{AtomicBool, Ordering}, Mutex}, thread, time::Duration
};

#[derive(Clone, Copy, Debug)]
struct Task {
    pub row_offset: i32,
}

static TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
static SPINNING: AtomicBool = AtomicBool::new(false);

/// Load a task or subtask with a spinner
#[macro_export]
macro_rules! start {
    ($($tokens:tt)*) => {
        $crate::__start_task__(format!($($tokens)*));
    };
}

/// Indicates that the most recently created task has passed and
/// replaces the spinner with a green check mark.
#[macro_export]
macro_rules! pass {
    ($($tokens:tt)*) => {
        $crate::__end_task__("\x1b[32;1m✔\x1b[0m", format!($($tokens)*));
    };
}

/// Indicates that the most recently created task has passed with a warning
/// and replaces the spinner with a hazard.
#[macro_export]
macro_rules! warn {
    ($($tokens:tt)*) => {
        $crate::__end_task__("\x1b[33;1m⚠\x1b[0m", format!($($tokens)*));
    };
}

/// Indicates that the most recently created task has failed
/// and replaces the spinner with a red x.
#[macro_export]
macro_rules! fail {
    ($($tokens:tt)*) => {
        $crate::__end_task__("\x1b[31;1m𝕩\x1b[0m", format!($($tokens)*))
    };
}

#[doc(hidden)]
pub fn __start_task__(message: String) {
    // this will never panic since mutex locks can only
    // fail if the thread holding the lock panics.
    // this is guarenteed as long as:
    //      1. TASKS is never locked outside of lytlog
    //      2. lytlog code never panics
    // so long as these two invariants are satisfied
    // (and they are by design) then locks of TASKS
    // will not panic.

    let mut tasks = TASKS.lock().unwrap();

    if tasks.len() > 0 {
        // adjust the offset (from bottom row) of each task
        for task in tasks.iter_mut() {
            task.row_offset += 1;
        }

        println!()
    }

    if let Some(last_row) = tasks.last().map(|task| task.row_offset) {
        print!("\x1b[s");

        if last_row > 1 {
            print!("\x1b[{}A\x1b[{}G┣", last_row - 1, (tasks.len() - 1) * 5 + 3)
        }

        for _ in 1..last_row {
            print!("\x1b[1D\x1b[1B┃")
        }

        print!("\x1b[u");
    }

    tasks.push(Task { row_offset: 0 });

    if tasks.len() > 1 {
        print!("{}", " ".repeat((tasks.len() - 2) * 5 + 2) + "┗━ ");
    }

    // attempts to print message, ignore if flush fails
    print!("\x1b[33;1m-\x1b[0m {message}");
    _ = io::stdout().flush();

    // atomically check if the spinner is running
    // if not then start the spinner.
    if SPINNING.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed) == Ok(false) {
        thread::spawn(spin);
    }
}

#[doc(hidden)]
pub fn __end_task__(symbol: &str, message: String) {
    let mut tasks = TASKS.lock().unwrap();

    if let Some(Task {row_offset: row}) = tasks.pop() {
        let column = tasks.len() * 5 + 1;

        // replace spinner with symbol:
        // \x1b[s           : save cursor's current position
        // \x1b[{row}A      : move the cursor up to correct row
        // \x1b[{column}G   : move the cursor to correct column
        // {symbol}         : print the symbol replacing the spinner
        // \x1b[K           : clear the current line
        // {message}        : print the ending message overwriting the old message.

        print!("\x1b[s");

        if row > 0 {
            print!("\x1b[{row}A");
        }

        print!("\x1b[{column}G{symbol} \x1b[K{message}");

        // restore the cursor's position if not the last task
        if row != 0 {
            print!("\x1b[u")
        }

        if tasks.len() == 0 {
            println!();
        }

        _ = io::stdout().flush();
    } else {
        // if not task is running, just print the symbol and message
        println!("{symbol} {message}");
    }

}

fn spin() {
    let mut spinner = '-';

    loop {
        let tasks = TASKS.lock().unwrap();

        // kill the thread if there are no more tasks
        if tasks.len() == 0 {
            break;
        }

        let mut column = 1;

        for Task { row_offset: row} in tasks.iter() {
            // replace the spinner with a new spinner:
            // \x1b[s               : save the cursor's current position
            // \x1b[{row}A          : move the cursor up to correct row
            // \x1b[{column}G       : move the cursor to the correct column
            // \x1b[33;1m           : set the foreground color to yellow and font to bold
            // {spinner}            : print the updated spinner character
            // \x1b[0m              : reset all formatting
            // \x1b[u               : restore saved cursor position

            print!("\x1b[s");

            if *row > 0 {
                print!("\x1b[{row}A ")
            }

            print!("\x1b[{column}G\x1b[33;1m{spinner}\x1b[0m\x1b[u");

            column += 5;
        }

        // most systems flush stdout by newlines
        // since no newlines were printed, we need
        // to flush stdout explicitly
        _ = io::stdout().flush();

        // update spinner to next spinner character (clockwise)
        spinner = match spinner {
            '-' => '\\',
            '\\' => '|',
            '|' => '/',
            '/' => '-',
            _ => '-', // This is not possible, but Rust demands it.
        };

        // drop tasks before the wait so other threads may use it.
        drop(tasks);

        // wait for 80ms; this can be changed to make the spinner go faster
        thread::sleep(Duration::from_millis(80));
    }

    // if the loop has ended, then the spinner has stopped and
    // will need to be restarted if another task starts
    SPINNING.store(false, Ordering::Relaxed);
}