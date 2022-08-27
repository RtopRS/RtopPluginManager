// Code based on the git2-rs example available on the URL: https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs.

use crate::util::utils::convert_to_readable_unity;
use colored::*;
use git2::{
    AnnotatedCommit, Commit, FetchOptions, Index, MergeAnalysis, MergePreference, Progress,
    Reference, RemoteCallbacks, Repository, Signature, Tree,
};
use std::io::{self, Write};
use std::str;

pub fn do_fetch<'a>(
    repo: &'a Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<AnnotatedCommit<'a>, git2::Error> {
    let mut cb: RemoteCallbacks = RemoteCallbacks::new();

    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                ":: {}",
                format!(
                    "Resolving deltas {}/{}\r",
                    stats.indexed_deltas(),
                    stats.total_deltas()
                )
            );
        } else if stats.total_objects() > 0 {
            let network_pct: usize = (100 * stats.received_objects()) / stats.total_objects();
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
            );
        }
        io::stdout().flush().unwrap();
        true
    });

    let mut fo: FetchOptions = FetchOptions::new();
    fo.remote_callbacks(cb);
    fo.download_tags(git2::AutotagOption::All);
    remote.fetch(refs, Some(&mut fo), None)?;

    let stats: Progress = remote.stats();
    println!(
        ":: {}",
        format!(
            "Received {}/{} objects for a total {}.",
            stats.indexed_objects(),
            stats.total_objects(),
            convert_to_readable_unity(stats.received_bytes() as f64)
        )
        .green()
    );
    let fetch_head: Reference = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut Reference,
    rc: &AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name: String = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg: String = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &AnnotatedCommit,
    remote: &AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree: Tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree: Tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor: Tree = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx: Index = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let msg: String = format!("Merge: {} into {}", remote.id(), local.id());
    let result_tree: Tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    let sig: Signature = repo.signature()?;
    let local_commit: Commit = repo.find_commit(local.id())?;
    let remote_commit: Commit = repo.find_commit(remote.id())?;
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    repo.checkout_head(None)?;
    Ok(())
}

pub fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    let analysis: (MergeAnalysis, MergePreference) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        let ref_name: String = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&ref_name) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                repo.reference(
                    &ref_name,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&ref_name)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        let head_commit: AnnotatedCommit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    }

    Ok(())
}
