// Code based on the git2-rs example available on the URL: https://github.com/rust-lang/git2-rs/blob/master/examples/clone.rs.

use crate::util::utils::convert_to_readable_unity;
use colored::*;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, Progress, RemoteCallbacks};
use std::cell::{RefCell, RefMut};
use std::io::{self, Write};
use std::path::Path;

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    newline: bool,
}

fn print(state: &mut State) {
    let stats: &Progress = state.progress.as_ref().unwrap();
    let network_pct: usize = (100 * stats.received_objects()) / stats.total_objects();
    if stats.received_objects() == stats.total_objects() {
        if !state.newline {
            print!("{}\r", " ".repeat(50));
            println!(":: {}", "Download terminated!".green());
            state.newline = true;
        }

        if state.total != 0 && state.current == state.total {
            println!(":: {}", "Checkout terminated!".green());
        } else {
            print!(
                ":: {}",
                format!("Checkout {}/{}...\r", state.current, state.total).green(),
            )
        }
    } else {
        print!(
            ":: {}",
            format!(
                "Download {}% - speed: {} - objects: {}/{}...\r",
                network_pct,
                convert_to_readable_unity(stats.received_bytes() as f64),
                stats.received_objects(),
                stats.total_objects()
            )
            .green(),
        )
    }
    io::stdout().flush().unwrap();
}

pub fn clone(url: String, path: &Path) {
    let state: RefCell<State> = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        newline: false,
    });
    let mut cb: RemoteCallbacks = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        print(&mut *state);
        true
    });

    let mut co: CheckoutBuilder = CheckoutBuilder::new();
    co.progress(|_, cur, total| {
        let mut state: RefMut<State> = state.borrow_mut();
        state.current = cur;
        state.total = total;
        print(&mut *state);
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(url.as_str(), path)
        .unwrap();
}
