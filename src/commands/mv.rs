use crate::commands::command::RunnablePerItemContext;
use crate::errors::ShellError;
use crate::parser::hir::SyntaxType;
use crate::parser::registry::{CommandRegistry, Signature};
use crate::prelude::*;
use std::path::PathBuf;

pub struct Move;

#[derive(Deserialize)]
pub struct MoveArgs {
    src: Tagged<PathBuf>,
    dst: Tagged<PathBuf>,
}

impl PerItemCommand for Move {
    fn name(&self) -> &str {
        "mv"
    }

    fn signature(&self) -> Signature {
        Signature::build("mv")
            .required("source", SyntaxType::Path)
            .required("destination", SyntaxType::Path)
            .named("file", SyntaxType::Any)
    }

    fn run(
        &self,
        call_info: &CallInfo,
        _registry: &CommandRegistry,
        shell_manager: &ShellManager,
        _input: Tagged<Value>,
    ) -> Result<VecDeque<ReturnValue>, ShellError> {
        call_info.process(shell_manager, mv)?.run()
    }
}

fn mv(
    MoveArgs { src, dst }: MoveArgs,
    RunnablePerItemContext {
        name,
        shell_manager,
    }: &RunnablePerItemContext,
) -> Result<VecDeque<ReturnValue>, ShellError> {
    let source = src.item.clone();
    let mut destination = dst.item.clone();
    let name_span = name;

    let sources: Vec<_> = match glob::glob(&source.to_string_lossy()) {
        Ok(files) => files.collect(),
        Err(_) => {
            return Err(ShellError::labeled_error(
                "Invalid pattern.",
                "Invalid pattern.",
                src.tag,
            ))
        }
    };

    if "." == destination.to_string_lossy() {
        destination = PathBuf::from(shell_manager.path());
    }

    let destination_file_name = {
        match destination.file_name() {
            Some(name) => PathBuf::from(name),
            None => {
                return Err(ShellError::labeled_error(
                    "Rename aborted. Not a valid destination",
                    "Rename aborted. Not a valid destination",
                    dst.span(),
                ))
            }
        }
    };

    if sources.len() == 1 {
        if let Ok(entry) = &sources[0] {
            let entry_file_name = match entry.file_name() {
                Some(name) => name,
                None => {
                    return Err(ShellError::labeled_error(
                        "Rename aborted. Not a valid entry name",
                        "Rename aborted. Not a valid entry name",
                        name_span,
                    ))
                }
            };

            if destination.exists() && destination.is_dir() {
                destination = match dunce::canonicalize(&destination) {
                    Ok(path) => path,
                    Err(e) => {
                        return Err(ShellError::labeled_error(
                            format!("Rename aborted. {:}", e.to_string()),
                            format!("Rename aborted. {:}", e.to_string()),
                            name_span,
                        ))
                    }
                };

                destination.push(entry_file_name);
            }

            if entry.is_file() {
                match std::fs::rename(&entry, &destination) {
                    Err(e) => {
                        return Err(ShellError::labeled_error(
                            format!(
                                "Rename {:?} to {:?} aborted. {:}",
                                entry_file_name,
                                destination_file_name,
                                e.to_string(),
                            ),
                            format!(
                                "Rename {:?} to {:?} aborted. {:}",
                                entry_file_name,
                                destination_file_name,
                                e.to_string(),
                            ),
                            name_span,
                        ));
                    }
                    Ok(o) => o,
                };
            }

            if entry.is_dir() {
                match std::fs::create_dir_all(&destination) {
                    Err(e) => {
                        return Err(ShellError::labeled_error(
                            format!(
                                "Rename {:?} to {:?} aborted. {:}",
                                entry_file_name,
                                destination_file_name,
                                e.to_string(),
                            ),
                            format!(
                                "Rename {:?} to {:?} aborted. {:}",
                                entry_file_name,
                                destination_file_name,
                                e.to_string(),
                            ),
                            name_span,
                        ));
                    }
                    Ok(o) => o,
                };
                #[cfg(not(windows))]
                {
                    match std::fs::rename(&entry, &destination) {
                        Err(e) => {
                            return Err(ShellError::labeled_error(
                                format!(
                                    "Rename {:?} to {:?} aborted. {:}",
                                    entry_file_name,
                                    destination_file_name,
                                    e.to_string(),
                                ),
                                format!(
                                    "Rename {:?} to {:?} aborted. {:}",
                                    entry_file_name,
                                    destination_file_name,
                                    e.to_string(),
                                ),
                                name_span,
                            ));
                        }
                        Ok(o) => o,
                    };
                }
                #[cfg(windows)]
                {
                    use crate::utils::FileStructure;

                    let mut sources: FileStructure = FileStructure::new();

                    sources.walk_decorate(&entry)?;

                    let strategy = |(source_file, depth_level)| {
                        let mut new_dst = destination.clone();

                        let path = dunce::canonicalize(&source_file)?;

                        let mut comps: Vec<_> = path
                            .components()
                            .map(|fragment| fragment.as_os_str())
                            .rev()
                            .take(1 + depth_level)
                            .collect();

                        comps.reverse();

                        for fragment in comps.iter() {
                            new_dst.push(fragment);
                        }

                        Ok((PathBuf::from(&source_file), PathBuf::from(new_dst)))
                    };

                    let sources = sources.paths_applying_with(strategy)?;

                    for (ref src, ref dst) in sources {
                        if src.is_dir() {
                            if !dst.exists() {
                                match std::fs::create_dir_all(dst) {
                                    Err(e) => {
                                        return Err(ShellError::labeled_error(
                                            format!(
                                                "Rename {:?} to {:?} aborted. {:}",
                                                entry_file_name,
                                                destination_file_name,
                                                e.to_string(),
                                            ),
                                            format!(
                                                "Rename {:?} to {:?} aborted. {:}",
                                                entry_file_name,
                                                destination_file_name,
                                                e.to_string(),
                                            ),
                                            name_span,
                                        ));
                                    }
                                    Ok(o) => o,
                                }
                            }
                        }

                        if src.is_file() {
                            match std::fs::rename(src, dst) {
                                Err(e) => {
                                    return Err(ShellError::labeled_error(
                                        format!(
                                            "Rename {:?} to {:?} aborted. {:}",
                                            entry_file_name,
                                            destination_file_name,
                                            e.to_string(),
                                        ),
                                        format!(
                                            "Rename {:?} to {:?} aborted. {:}",
                                            entry_file_name,
                                            destination_file_name,
                                            e.to_string(),
                                        ),
                                        name_span,
                                    ));
                                }
                                Ok(o) => o,
                            }
                        }
                    }

                    match std::fs::remove_dir_all(entry) {
                        Err(e) => {
                            return Err(ShellError::labeled_error(
                                format!(
                                    "Rename {:?} to {:?} aborted. {:}",
                                    entry_file_name,
                                    destination_file_name,
                                    e.to_string(),
                                ),
                                format!(
                                    "Rename {:?} to {:?} aborted. {:}",
                                    entry_file_name,
                                    destination_file_name,
                                    e.to_string(),
                                ),
                                name_span,
                            ));
                        }
                        Ok(o) => o,
                    };
                }
            }
        }
    } else {
        if destination.exists() {
            if !sources.iter().all(|x| {
                if let Ok(entry) = x.as_ref() {
                    entry.is_file()
                } else {
                    false
                }
            }) {
                return Err(ShellError::labeled_error(
                    "Rename aborted (directories found). Renaming in patterns not supported yet (try moving the directory directly)",
                    "Rename aborted (directories found). Renaming in patterns not supported yet (try moving the directory directly)",
                    src.tag,
                ));
            }

            for entry in sources {
                if let Ok(entry) = entry {
                    let entry_file_name = match entry.file_name() {
                        Some(name) => name,
                        None => {
                            return Err(ShellError::labeled_error(
                                "Rename aborted. Not a valid entry name",
                                "Rename aborted. Not a valid entry name",
                                name_span,
                            ))
                        }
                    };

                    let mut to = PathBuf::from(&destination);
                    to.push(entry_file_name);

                    if entry.is_file() {
                        match std::fs::rename(&entry, &to) {
                            Err(e) => {
                                return Err(ShellError::labeled_error(
                                    format!(
                                        "Rename {:?} to {:?} aborted. {:}",
                                        entry_file_name,
                                        destination_file_name,
                                        e.to_string(),
                                    ),
                                    format!(
                                        "Rename {:?} to {:?} aborted. {:}",
                                        entry_file_name,
                                        destination_file_name,
                                        e.to_string(),
                                    ),
                                    name_span,
                                ));
                            }
                            Ok(o) => o,
                        };
                    }
                }
            }
        } else {
            return Err(ShellError::labeled_error(
                format!("Rename aborted. (Does {:?} exist?)", destination_file_name),
                format!("Rename aborted. (Does {:?} exist?)", destination_file_name),
                dst.span(),
            ));
        }
    }

    Ok(VecDeque::new())
}