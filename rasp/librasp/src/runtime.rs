use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use anyhow::{anyhow, Result};
use log::*;
use serde_json;
use std::fs;
use std::path::Path;
use crate::cpython;
use crate::golang::golang_bin_inspect;
use crate::jvm::vm_version;
use crate::nodejs::nodejs_version;
use crate::php::{inspect_phpfpm, inspect_phpfpm_version, inspect_phpfpm_zts};
use crate::process::ProcessInfo;
use serde::{Deserialize, Serialize};

const DEFAULT_JVM_FILTER_JSON_STR: &str = r#"{"exe": ["java"]}"#;
const DEFAULT_CPYTHON_FILTER_JSON_STR: &str = r#"{"exe": ["python","python2", "python3","python2.7", "python3.4", "python3.5", "python3.6", "python3.7", "python3.8", "python3.9", "python3.10", "uwsgi"]}"#;
const DEFAULT_NODEJS_FILTER_JSON_STR: &str = r#"{"exe": ["node", "nodejs"]}"#;

impl RuntimeInspect for ProcessInfo {}

#[derive(Debug, Clone)]
pub struct Runtime {
    pub name: &'static str,
    pub version: String,
    pub size: u64,
}

impl Display for Runtime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.name, self.version, self.size)
    }
}

pub enum ProbeState {
    Attached,
    NotAttach,
    AttachedVersionNotMatch,
}

pub trait ProbeStateInspect {
    fn inspect_pid(pid: i32) -> Result<ProbeState> {
        let process_info = ProcessInfo::from_pid(pid)?;
        // inspect process
        Self::inspect_process(&process_info)
    }
    fn inspect_process(process_info: &ProcessInfo) -> Result<ProbeState>;
}

pub trait ProbeCopy {
    fn names() -> (Vec<String>, Vec<String>);
}

pub trait RuntimeInspect {
    fn check_jvm_attach_mechanism(pid: i32) -> Result<bool, std::io::Error> {
        let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", pid))?;
        Ok(cmdline.contains("+DisableAttachMechanism"))
    }
    
    fn check_signal_dispatch(pid: i32) -> Result<bool, std::io::Error> {
        let task_dir = format!("/proc/{}/task", pid);
        for entry in fs::read_dir(&task_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
    
            let comm_file = format!("{}/comm", entry.path().display());
            //info!("pid {},comm file : {}", pid, comm_file);
            if !Path::new(&comm_file).exists() {
                continue; // Skip if 'comm' file doesn't exist
            }
    
            let comm = fs::read_to_string(comm_file)?;
            //info!("pid {}, comm: {}", pid, comm);
            if comm.contains("Signal Dispatch") || comm.contains("Attach Listener") {
                return Ok(true);
            }
        }
    
        Ok(false)
    }

    fn inspect_from_process_info(process_info: &mut ProcessInfo) -> Result<Option<Runtime>> {
        let process_exe_file = process_info.exe_name.clone().unwrap();
        debug!("runtime inspect: exe file: {}", process_exe_file);
        let jvm_process_filter: RuntimeFilter =
            match serde_json::from_str(DEFAULT_JVM_FILTER_JSON_STR) {
                Ok(jvm_filter) => jvm_filter,
                Err(e) => {
                    error!("filter deserialize failed: {}", e);
                    return Err(anyhow!("jvm filter deserialize failed: {}", e));
                }
            };
        let jvm_process_filter_check_reuslt =
            match jvm_process_filter.match_exe(&process_exe_file) {
                Ok(o) => o,
                Err(_) => false,
            };
        
        if  jvm_process_filter_check_reuslt {
            match Self::check_jvm_attach_mechanism(process_info.pid) {
                Ok(v) => {
                    if v == true {
                        let msg = format!("npid: {}, set DisableAttachMechanism, so can not attach", process_info.pid);
                        info!("pid: {}, set DisableAttachMechanism, so can not attach", process_info.pid);
                        return Err(anyhow!(msg));
                    }
                }
                Err(e) => info!("Failed to check '+DisableAttachMechanism': {}", e),
            }
            
            // https://bugs.openjdk.org/browse/JDK-8292695
            // let uptime = count_uptime(process_info.start_time.unwrap()).unwrap_or(0);
            // if uptime > 0  && uptime < 5 {
            //     let interval = 5 - uptime;
            //     info!("JVM process {} just start, so sleep {} sec", process_info.pid, interval);
            //     std::thread::sleep(Duration::from_secs(interval));
            // }
            match Self::check_signal_dispatch(process_info.pid) {
                Ok(v) => {
                    if v == true {
                        let version = match vm_version(process_info.pid) {
                            Ok(ver) => {
                                if ver < 8 {
                                    warn!("process {} Java version lower than 8: {}, so not inject", process_info.pid, ver);
                                    let msg = format!("Java version lower than 8: {}, so not inject", ver);
                                    return Err(anyhow!(msg));
                                } else if ver == 13 || ver == 14 {
                                    // jdk bug https://bugs.openjdk.org/browse/JDK-8222005
                                    warn!("process {} Java version {} has attach bug, so not inject", process_info.pid, ver);
                                    let msg = format!("process {} Java version {} has attach bug, so not inject", process_info.pid, ver);
                                    return Err(anyhow!(msg));
                                }
                                ver.to_string()
                            }
                            Err(e) => {
                                warn!("read jvm version failed: {}", e);
                                String::new()
                            }
                        };
                        return Ok(Some(Runtime {
                            name: "JVM",
                            version: version,
                            size: 0,
                        }));
                    } else {
                        warn!("pid: {} not found Signal Dispatcher, so not report version", process_info.pid);
                        return Ok(Some(Runtime {
                            name: "JVM",
                            version: String::new(),
                            size: 0,
                        }));
                    }
                }
                Err(e) => info!("pid: {}, Failed to check 'Signal Dispatcher': {}", process_info.pid, e),
            }
        }
        let cpython_process_filter: RuntimeFilter =
            match serde_json::from_str(DEFAULT_CPYTHON_FILTER_JSON_STR) {
                Ok(cpython_filter) => cpython_filter,
                Err(e) => {
                    error!("filter deserialize failed: {}", e);
                    return Err(anyhow!("cpython filter deserialize failed: {}", e));
                }
            };
        let process_filter_check_result = match cpython_process_filter.match_exe(&process_exe_file)
        {
            Ok(o) => o,
            Err(_) => false,
        };
        if process_filter_check_result {
            return Ok(Some(Runtime {
                name: "CPython",
                version: String::new(),
                size: 0,
            }));
        }
        let nodejs_process_filter: RuntimeFilter =
            match serde_json::from_str(DEFAULT_NODEJS_FILTER_JSON_STR) {
                Ok(nodejs_filter) => nodejs_filter,
                Err(e) => {
                    error!("filter deserialize failed: {}", e);
                    return Err(anyhow!("cpython filter deserialize failed: {}", e));
                }
            };
        let nodejs_process_filter_check_reuslt =
            match nodejs_process_filter.match_exe(&process_exe_file) {
                Ok(o) => o,
                Err(_) => false,
            };
        if nodejs_process_filter_check_reuslt {
            let version = match nodejs_version(process_info.pid, &process_exe_file) {
                Ok((major, minor, v)) => {
                    if major < 8 {
                        let msg = format!("nodejs version lower than 8.6: {}", v);
                        return Err(anyhow!(msg));
                    }
                    if major == 8 {
                        if minor < 6 {
                            let msg = format!("nodejs version lower than 8.6: {}", v);
                            return Err(anyhow!(msg));
                        }
                    }
                    v
                }
                Err(e) => {
                    warn!("read nodejs version failed: {}", e);
                    String::new()
                }
            };
            return Ok(Some(Runtime {
                name: "NodeJS",
                version,
                size: 0,
            }));
        }
        let pid = process_info.pid.clone();
        let exe_path = process_info.exe_path.clone().unwrap().clone();
        // /proc/<pid>/<exe_path> for process in container
        let mut path = PathBuf::from(format!("/proc/{}/root/", pid));
        let exe_path_buf = PathBuf::from(exe_path);
        if !exe_path_buf.has_root() {
            path.push(exe_path_buf);
        } else {
            for p in exe_path_buf.iter() {
                if p == std::ffi::OsString::from("/") {
                    continue;
                }
                path.push(p);
            }
        }
        match golang_bin_inspect(path) {
            Ok(res) => {
                if res > 0 {
                    return Ok(Some(Runtime {
                        name: "Golang",
                        version: String::new(),
                        size: res,
                    }));
                }
            }
            Err(e) => {
                warn!("detect golang bin failed: {}", e.to_string());
            }
        };
        match inspect_phpfpm(&process_info) {
            Ok(result) => {
                if result {
                    match inspect_phpfpm_version(&process_info) {
                        Ok(version) => {
                            if inspect_phpfpm_zts(&process_info)? {
                                return Ok(Some(Runtime {
                                    name: "PHP",
                                    version: format!("{}.zts", version),
                                    size: 0,
                                }));
                            } else {
                                return Ok(Some(Runtime {
                                    name: "PHP",
                                    version: version,
                                    size: 0,
                                }));
                            }
                        }
                        Err(e) => {
                            warn!("detect php-fpm version failed: {}", e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                warn!("detect phpfpm bin failed: {}", e.to_string());
            }
        }
        match cpython::CPythonRuntime::python_inspect(&process_info) {
            Some(version) => {
                return Ok(Some(Runtime {
                    name: "CPython",
                    version,
                    size: 0,
                }))
            }
            None => {}
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeFilter {
    pub cmdline: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
    pub exe: Option<Vec<String>>,
    pub runtime: Option<String>,
}

impl RuntimeFilter {
    pub fn new() -> Self {
        Self {
            cmdline: None,
            env: None,
            exe: None,
            runtime: None,
        }
    }

    pub fn defaut() -> Self {
        RuntimeFilter::new()
    }

    pub fn update_runtime(mut self, runtime_name: String) -> Self {
        self.runtime = Some(runtime_name);
        self
    }
    pub fn match_runtime(&self, runtime: &str) -> Result<bool, ()> {
        if let Some(rt) = &self.runtime {
            if rt == runtime {
                return Ok(true);
            } else {
                return Ok(false);
            }
        } else {
            return Ok(true);
        }
    }

    pub fn add_env_filter(mut self, env_name: String, env_value: String) -> Self {
        let env_vec = match self.env {
            Some(mut env_origin_vec) => {
                env_origin_vec.push((env_name, env_value));
                env_origin_vec
            }
            None => {
                let mut new_vec = Vec::new();
                new_vec.push((env_name, env_value));
                new_vec
            }
        };
        self.env = Some(env_vec);
        self
    }

    pub fn add_cmdline_filter(mut self, cmd_line_keyword: String) -> Self {
        let cmdline_vec = match self.cmdline {
            Some(mut cmdline_origin_vec) => {
                cmdline_origin_vec.push(cmd_line_keyword);
                cmdline_origin_vec
            }
            None => {
                let mut new_vec = Vec::new();
                new_vec.push(cmd_line_keyword);
                new_vec
            }
        };
        self.cmdline = Some(cmdline_vec);
        self
    }

    pub fn add_exe_filter(mut self, exe_full_match_word: String) -> Self {
        let exe_vec = match self.exe {
            Some(mut exe_origin_vec) => {
                exe_origin_vec.push(exe_full_match_word);
                exe_origin_vec
            }
            None => {
                let mut new_vec = Vec::new();
                new_vec.push(exe_full_match_word);
                new_vec
            }
        };
        self.exe = Some(exe_vec);
        self
    }
    pub fn match_exe(&self, target_exe: &String) -> Result<bool, ()> {
        if self.exe.is_none() {
            return Ok(false);
        }
        let exe_vec = self.exe.as_ref().unwrap();
        for exe in exe_vec.iter() {
            if target_exe == exe {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn match_cmdline(&self, target_cmdline: &String) -> Result<bool, ()> {
        if self.cmdline.is_none() {
            return Ok(true);
        }
        let cmdlines = self.cmdline.as_ref().unwrap();
        for cmdline in cmdlines.iter() {
            if target_cmdline.starts_with(cmdline) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn match_env(&self, target_env: &HashMap<OsString, OsString>) -> Result<bool, ()> {
        if self.env.is_none() {
            return Ok(true);
        }
        let mut count = 0;
        let envs = self.env.as_ref().unwrap();
        let env_count = envs.len();
        for env in envs.iter() {
            if let Some(target_env_value) = target_env.get(&OsString::from(env.0.clone())) {
                if (*target_env_value) == OsString::from(env.1.clone()) {
                    count += 1;
                } else {
                    return Ok(false);
                }
            }
        }
        if count == env_count {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn match_process_info(
        &self,
        cmdline: &String,
        environ: &HashMap<OsString, OsString>,
        runtime_name: &String,
    ) -> Result<bool, ()> {
        if self.cmdline.is_none() && self.env.is_none() {
            return Err(());
        }
        let runtime_match_result = match self.match_runtime(runtime_name) {
            Ok(b) => b,
            Err(_) => false,
        };
        //debug!("runtime match: {:?}", &runtime_match_result);
        let cmdline_match_result = match self.match_cmdline(cmdline) {
            Ok(b) => b,
            Err(_) => false,
        };
        //debug!("cmdline match: {:?}", &cmdline_match_result);
        let env_match_result = match self.match_env(environ) {
            Ok(b) => b,
            Err(_) => false,
        };
        //debug!("env match: {:?}", &env_match_result);
        if cmdline_match_result && env_match_result && runtime_match_result {
            return Ok(true);
        }
        return Ok(false);
    }
}
