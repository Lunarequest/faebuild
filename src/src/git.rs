use super::super::recipe::Sources;
use git2::{Oid, Repository};
use std::{path::PathBuf, process::exit, str::FromStr};

pub fn git_solver(sources: Sources) {
    //verify required feild exists
    let commit = match sources.commit {
        Some(ref commit) => commit,
        None => {
            eprintln!("A commit is required.");
            exit(1);
        }
    };
    let url = match sources.url {
        Some(url) => url,
        None => {
            eprintln!("Git type source requires a url");
            exit(1);
        }
    };

    let url_vec = url.split("/").collect::<Vec<&str>>();
    let repo_dir = url_vec[url_vec.len() - 1].split(".").collect::<Vec<&str>>()[0];
    let path = PathBuf::from_str(&repo_dir).unwrap();
    let recursive = match sources.recursive {
        Some(rec) => rec,
        None => false,
    };
    let repo = if path.exists() {
        let repo = Repository::open(repo_dir).expect("unable to open git repository");
        repo
    } else {
        let repo = if recursive == true {
            Repository::clone_recurse(url.as_str(), repo_dir).expect("could not clong repo")
        } else {
            Repository::clone(url.as_str(), repo_dir).expect("could not clong repo")
        };
        repo
    };
    match sources.tag {
        Some(tag) => {
            println!("Opp a tag")
        }
        None => {
            let oid = Oid::from_str(&commit).unwrap();
            let commit_git = match repo.find_commit(oid) {
                Ok(commit) => commit,
                Err(_e) => {
                    eprintln!("Unable to find commit {commit} exiting");
                    exit(1);
                }
            };
            let _branch = repo.branch(&commit, &commit_git, false).unwrap();
            let obj = repo
                .revparse_single(&("refs/heads/".to_owned() + &commit))
                .unwrap();
            repo.checkout_tree(&obj, None).expect("");

            repo.set_head(&("refs/heads/".to_owned() + &commit))
                .expect("");
        }
    }
}
