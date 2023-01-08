use git2::{Reference, Repository};
use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str::FromStr;
use std::string::ToString;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, ToString};

pub struct Git {
    pub repo: Option<Repository>,
}
impl Git {
    pub fn new() -> Self {
        let mut e = Git { repo: None };
        e.exists();
        return e;
    }
    ///  Detect if there is a git repo in pwd
    fn exists(&mut self) -> bool {
        // Seek git repo in current directory
        let root = env::current_dir().unwrap();
        let repo = Repository::discover(root).unwrap();
        // Set working dir
        let exist = repo.workdir().is_some();
        if exist {
            let wd = repo.workdir().unwrap().display().to_string();
            // Set working directory to .git parent
            // Use this function to teleport from hook folder to root
            // and read config file
            env::set_current_dir(wd).unwrap();
        }
        self.repo = Some(repo);
        return exist;
    }
    /// Return actual attached branch
    pub fn get_branch(&self) -> Result<String, Box<dyn Error>> {
        // Only tested on attached HEAD
        // No edge case when head is a commit or else...
        let repo = self.repo.as_ref().unwrap();
        let head = repo.head()?;
        let name = head.shorthand().unwrap().to_owned();
        Ok(name)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, EnumString, ToString, EnumIter)]
pub enum GitHook {
    ApplypatchMsg,
    PreApplypatch,
    PostApplypatch,
    PreCommit,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PreRebase,
    PostCheckout,
    PostMerge,
    PreReceive,
    Update,
    PostReceive,
    PostUpdate,
    PreAutoGc,
    PostRewrite,
    PrePush,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Hook {
    githook: GitHook,
}
impl Hook {
    pub fn to_string(&self) -> String {
        let string = self.githook.to_string().replace("_", "-");
        return string;
    }
    /// Convert str into enum GitHook
    pub fn from_str(name: &str) -> Hook {
        let githook = GitHook::from_str(&name.replace("_", "-")).unwrap();
        return Hook { githook: githook };
    }
    /// Detect name of the hook that triggers script
    pub fn origin() -> Result<Hook, Box<dyn Error>> {
        let root = env::current_dir()?;
        let path_string = root.display().to_string();
        let my_bool = path_string.contains("/.git/hooks/");
        let name = root
            .parent()
            .unwrap()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        println!("{}", name);
        let hook = Hook::from_str(name);
        Ok(hook)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Hooks {}
impl Hooks {
    pub fn iter() -> GitHookIter {
        GitHook::iter()
    }
    /// Create/Ensure git hooks file trees
    pub fn ensure() -> Result<(), Box<dyn Error>> {
        let root = ".git/hooks";
        let extension = ".d";
        let bin = "pipelight-trigger";

        let bin_path = format!("/usr/bin/{}", bin);
        let bin_path = Path::new(&bin_path);
        for hook in GitHook::iter() {
            let file = format!("{}/{}", root, hook.to_string());
            let file = Path::new(&file);

            let dir = format!("{}/{}{}", root, hook.to_string(), extension);
            let dir = Path::new(&dir);

            let link = format!("{}/{}", dir.display(), bin);
            let link = Path::new(&link);

            Hooks::ensure_hook(file, &hook)?;
            Hooks::ensure_directory(dir)?;
            Hooks::ensure_symlink(bin_path, link)?;
        }
        Ok(())
    }
    /// Create directories
    fn ensure_directory(path: &Path) -> Result<(), Box<dyn Error>> {
        let dir_exists = path.exists();
        if dir_exists {
            fs::remove_dir_all(path)?;
        }
        fs::create_dir(path)?;
        Ok(())
    }
    /// Create a hook that will call subfolder script
    fn ensure_hook(path: &Path, hook: &GitHook) -> Result<(), Box<dyn Error>> {
        let exists = path.exists();
        if exists {
            fs::remove_file(path)?;
        }
        let file = fs::File::create(path)?;
        let metadata = file.metadata()?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;

        Hooks::write(path, hook)?;
        Ok(())
    }
    fn write(path: &Path, hook: &GitHook) -> Result<(), Box<dyn Error>> {
        let git = Git::new();
        let root = git.repo.unwrap().path().display().to_string();
        let mut file = fs::File::create(path)?;
        let s = format!(
            "#!/bin/sh \n\
                dir=\"{root}hooks/{hook}.d\" \n\
                for f in \"$dir\"[>; do \n\
                  \"$f\" {hook}\n\
                done",
            root = root,
            hook = hook.to_string()
        );
        let b = s.as_bytes();
        file.write_all(b)?;
        Ok(())
    }
    fn ensure_symlink(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
        let link_exists = dest.exists();
        if link_exists {
            fs::remove_file(dest)?;
        }
        symlink(src, dest)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn internal() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}