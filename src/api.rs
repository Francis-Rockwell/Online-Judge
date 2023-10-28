use super::{CONFIG, CONTEST_LIST, JOB_LIST, MYSQL, USER_LIST};
use crate::{
    config::{fread, Config, Error, ProblemCase, ProblemType},
    structs::{
        later, string2result, string2state, Case, Contest, ContestArgs, JobArgs, JobRequest,
        JobResponse, MyResult, State, User,
    },
};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, Utc};
use log;
use mysql::prelude::*;
use mysql::*;
use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;

#[get("/hello/{name}")]
pub async fn greet(name: web::Path<String>) -> impl Responder {
    log::info!(target: "greet_handler", "Greeting {}", name);
    format!("Hello {name}!")
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
pub async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}
/*
    function: to check the ans and out in standard mode, along with an empty info
    input: ans: a &str of the path of answer
    output: if standardly the same, Some(true), otherwise, Some(false). All with empty info.
*/
async fn standard_check(ans: &str) -> (Option<bool>, String) {
    let outtext = fread("tmpdir/test.out", "outtext").unwrap();
    let anstext = fread(ans, "ansfile").unwrap();
    let outtext = outtext.trim().split("\n").collect::<Vec<&str>>();
    let anstext = anstext.trim().split("\n").collect::<Vec<&str>>();
    if outtext.len() != anstext.len() {
        return (Some(false), "".to_string());
    } else {
        for i in 0..anstext.len() {
            let out = outtext[i].trim();
            let ans = anstext[i].trim();
            if out != ans {
                return (Some(false), "".to_string());
            }
        }
    }
    (Some(true), "".to_string())
}
/*
    function: to check the ans and out in strict mode, along with an empty info
    input: ans: a &str of the path of answer
    output: if strictly the same, Some(true), otherwise, Some(false). All with empty info.
*/
async fn strict_check(ans: &str) -> (Option<bool>, String) {
    let outtext = fread("tmpdir/test.out", "outtext").unwrap();
    let anstext = fread(ans, "ansfile").unwrap();
    if outtext != anstext {
        (Some(false), "".to_string())
    } else {
        (Some(true), "".to_string())
    }
}
/*
    function: to check the ans and out in special judge mode, along with certain info
    input: ans: a &str of the path of answer
           path: a vec of String from misc related to the special judge
    output: if Accepted, Some(true), if Wrong Answer, Some(false), otherwise, None. All with certain info.
*/
async fn special_judge(ans: &str, spj: Vec<String>) -> (Option<bool>, String) {
    let out_file = File::create("tmpdir/spj.out").expect("Fail to create spjout_file");
    let err_file = File::create("tmpdir/spj.err").expect("Fail to create spjerr_file");
    Command::new(spj[0].clone())
        .args([
            spj[1].clone(),
            "tmpdir/test.out".to_string(),
            ans.to_string(),
        ])
        .stdout(Stdio::from(out_file))
        .stderr(Stdio::from(err_file))
        .output()
        .expect("Fail to spj");
    let outtext = fread("tmpdir/spj.out", "spjout").unwrap();
    let errtext = fread("tmpdir/spj.err", "spjerr").unwrap();
    let outtext = outtext.trim().split("\n").collect::<Vec<&str>>();
    if errtext.is_empty() && outtext.len() == 2 {
        match outtext[0] {
            "Accepted" => return (Some(true), outtext[1].to_string()),
            "Wrong Answer" => return (Some(false), outtext[1].to_string()),
            _ => return (None, outtext[1].to_string()),
        }
    } else {
        return (None, "".to_string());
    }
}
/*
    function: to check the ans and out, along with certain info
    input: ptype: a &ProblemType of the problem's type
           ans: a &str of the path of answer
           spj: a vec of String from misc related to the special judge
    output: if Accepted, Some(true), if Wrong Answer, Some(false), otherwise, None, which can only occur in special judge.
            String will be the info.
*/
async fn check(ptype: &ProblemType, ans: &str, spj: Option<Vec<String>>) -> (Option<bool>, String) {
    match ptype {
        ProblemType::Standard => standard_check(ans).await,
        ProblemType::Strict => strict_check(ans).await,
        ProblemType::Spj => special_judge(ans, spj.unwrap()).await,
        ProblemType::DynamicRanking => standard_check(ans).await,
    }
}
/*
    function: to compile a rust programme
    input: code: a &str of the source code
    output: case0
*/
async fn rust_compile(code: &str) -> Case {
    Command::new("mkdir")
        .arg("tmpdir")
        .output()
        .expect("Fail to mkdir");
    let mut main = File::create("tmpdir/main.rs").expect("Fail to create main.rs");
    main.write_all(code.as_bytes())
        .expect("write main.rs failed");
    let result: MyResult;
    let err_file = File::create("tmpdir/compile.err").expect("Fail to create compile_err_file");
    let now = Instant::now();
    Command::new("rustc")
        .args(["-C", "opt-level=2", "tmpdir/main.rs", "-o", "tmpdir/test"])
        .stderr(Stdio::from(err_file))
        .output()
        .expect("Fail to try compile");
    let time = now.elapsed().as_micros();
    if fread("tmpdir/compile.err", "compile_err_file")
        .unwrap()
        .is_empty()
    {
        result = MyResult::CS;
    } else {
        result = MyResult::CE;
    }
    Case {
        id: 0,
        result,
        time,
        memory: 0.0,
        info: String::from(""),
    }
}
/*
    function: to compile a cpp programme
    input: code: a &str of the source code
    output: case0
*/
async fn cpp_compile(code: &str) -> Case {
    Command::new("mkdir")
        .arg("tmpdir")
        .output()
        .expect("Fail to mkdir");
    let mut main = File::create("tmpdir/main.cpp").expect("Fail to create main.cpp");
    main.write_all(code.as_bytes())
        .expect("write main.cpp failed");
    let result: MyResult;
    let err_file = File::create("tmpdir/compile.err").expect("Fail to create compile_err_file");
    let now = Instant::now();
    Command::new("g++")
        .args(["-O3", "tmpdir/main.cpp", "-o", "tmpdir/test"])
        .stderr(Stdio::from(err_file))
        .output()
        .expect("Fail to try compile");
    let time = now.elapsed().as_micros();
    if fread("tmpdir/compile.err", "compile_err_file")
        .unwrap()
        .is_empty()
    {
        result = MyResult::CS;
    } else {
        result = MyResult::CE;
    }
    Case {
        id: 0,
        result,
        time,
        memory: 0.0,
        info: String::from(""),
    }
}
/*
    function: to compile a c programme
    input: code: a &str of the source code
    output: case0
*/
async fn c_compile(code: &str) -> Case {
    Command::new("mkdir")
        .arg("tmpdir")
        .output()
        .expect("Fail to mkdir");
    let mut main = File::create("tmpdir/main.c").expect("Fail to create main.c");
    main.write_all(code.as_bytes())
        .expect("write main.c failed");
    let result: MyResult;
    let err_file = File::create("tmpdir/compile.err").expect("Fail to create compile_err_file");
    let now = Instant::now();
    Command::new("gcc")
        .args(["-O3", "tmpdir/main.c", "-o", "tmpdir/test"])
        .stderr(Stdio::from(err_file))
        .output()
        .expect("Fail to try compile");
    let time = now.elapsed().as_micros();
    if fread("tmpdir/compile.err", "compile_err_file")
        .unwrap()
        .is_empty()
    {
        result = MyResult::CS;
    } else {
        result = MyResult::CE;
    }
    Case {
        id: 0,
        result,
        time,
        memory: 0.0,
        info: String::from(""),
    }
}
/*
    function: to test a case of a porblem
    input: case: a &ProblemCase of case to be tested
           id: a usize of the case'id
           ptype: a &ProblemType of the problem's type
           spj: a vec of String from misc related to the special judge
    output: a Case
*/
async fn testcase(
    case: &ProblemCase,
    id: usize,
    ptype: &ProblemType,
    spj: Option<Vec<String>>,
) -> Case {
    let in_file = File::open(case.input_file.clone()).expect("Fail to create out_file");
    let out_file = File::create("tmpdir/test.out").expect("Fail to create out_file");
    let err_file = File::create("tmpdir/test.err").expect("Fail to create err_file");
    let result: MyResult;
    let mut time = 0;
    let limit = Duration::from_micros(case.time_limit as u64);
    let mut info = "".to_string();
    let now = Instant::now();
    let mut child = Command::new("tmpdir/test")
        .stdin(Stdio::from(in_file))
        .stdout(Stdio::from(out_file))
        .stderr(Stdio::from(err_file))
        .spawn()
        .unwrap();
    match child
        .wait_timeout(limit + Duration::from_millis(500))
        .unwrap()
    {
        // check if the case exceed the time limit
        Some(_) => {
            if fread("tmpdir/test.err", "err_file").unwrap().is_empty() {
                // test.err is empty
                let check = check(ptype, &case.answer_file, spj).await;
                info = check.1;
                if check.0.is_none() {
                    // none stands for special judge error
                    result = MyResult::SPJE;
                } else {
                    if check.0.unwrap() {
                        // Some(true)
                        time = now.elapsed().as_micros();
                        if time > case.time_limit {
                            result = MyResult::TLE;
                        } else {
                            result = MyResult::Accepted;
                        }
                    } else {
                        // Some(false)
                        result = MyResult::WA;
                    }
                }
            } else {
                // runtime error
                result = MyResult::RE;
            }
        }
        None => {
            // exceed limit plus duration
            result = MyResult::TLE;
        }
    };
    Case {
        id,
        result,
        time,
        memory: 0.0,
        info,
    }
}
/*
    function: to test a JobRequest
    input: req: a &JobRequest of this request
           config: a &Config of the global config
           jobid: a usize of this job's id
    output: a JobResponse
*/
async fn process_post(req: &JobRequest, config: &Config, jobid: usize) -> JobResponse {
    let mut sub = 0;
    for i in 0..config.problems.len() {
        if config.problems[i].id == req.problem_id {
            sub = i;
            break;
        }
    }
    // find the order of the problem in config as sub
    let ptype = &config.problems[sub].ty;
    let mut score = 0.0;
    let mut result = MyResult::Accepted;
    let created_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let case0 = match req.language.as_str() {
        "Rust" => rust_compile(&req.source_code).await,
        "C++" => cpp_compile(&req.source_code).await,
        "C" => c_compile(&req.source_code).await,
        _ => panic!("unexpected language"),
    }; // the panic will never occur since the language was checked in post_job
    let mut cases: Vec<Case> = vec![case0];
    if cases[0].result != MyResult::CS {
        // when compile err, all cases turn to waiting
        for i in 0..config.problems[sub].cases.len() {
            result = MyResult::CE;
            cases.push(Case {
                id: i + 1,
                result: MyResult::Waiting,
                time: 0,
                memory: 0.0,
                info: String::from(""),
            });
        }
    } else {
        // compile successs
        let misc = config.problems[sub].misc.as_ref();
        let spj: Option<Vec<String>>;
        if misc.is_some() && misc.unwrap().special_judge.is_some() {
            spj = Some(misc.unwrap().special_judge.clone().unwrap());
        } else {
            spj = None;
        }
        if misc.is_some() && misc.unwrap().packing.is_some() {
            // if its packed judging
            let pack = misc.unwrap().packing.as_ref().unwrap();
            let mut count = 0;
            for i in 0..pack.len() {
                let mut judge = true;
                let mut packscore = 0.0;
                for j in 0..pack[i].len() {
                    let problem_case = &config.problems[sub].cases[count + j];
                    if judge {
                        // the cases before was all accepted
                        let case = testcase(problem_case, count + j + 1, ptype, spj.clone()).await;
                        if case.result == MyResult::Accepted {
                            // this case is accepted, add score
                            if config.problems[sub].ty == ProblemType::DynamicRanking {
                                // if dynamic ranking, the score should be altered
                                let drr = config.problems[sub]
                                    .misc
                                    .as_ref()
                                    .unwrap()
                                    .dynamic_ranking_ratio
                                    .unwrap();
                                packscore += problem_case.score * (1.0 - drr);
                            } else {
                                packscore += problem_case.score;
                            }
                            cases.push(case);
                        } else {
                            // this case is not accepted
                            result = case.result.clone();
                            judge = false;
                            cases.push(case);
                        }
                    } else {
                        // there was a case not accepted in this pack
                        cases.push(Case {
                            id: count + j + 1,
                            result: MyResult::Skipped,
                            time: 0,
                            memory: 0.0,
                            info: String::from(""),
                        });
                    }
                }
                if judge {
                    score += packscore;
                }
                count += pack[i].len(); // count stands for the number of cases before this pack
            }
        } else {
            // not packed judging
            for i in 0..config.problems[sub].cases.len() {
                let problem_case = &config.problems[sub].cases[i];
                let post_case = testcase(problem_case, i + 1, ptype, spj.clone()).await;
                if post_case.result == MyResult::Accepted {
                    // this case is accepted, add score
                    if config.problems[sub].ty == ProblemType::DynamicRanking {
                        // if dynamic ranking, the score should be altered
                        let drr = config.problems[sub]
                            .misc
                            .as_ref()
                            .unwrap()
                            .dynamic_ranking_ratio
                            .unwrap();
                        score += problem_case.score * (1.0 - drr);
                    } else {
                        score += problem_case.score;
                    }
                    cases.push(post_case);
                } else {
                    // this case is not accepted
                    result = post_case.result.clone();
                    cases.push(post_case);
                }
            }
        }
    }
    Command::new("rm")
        .args(["-rf", "tmpdir"])
        .output()
        .expect("Fail to delete tmpdir");
    let updated_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let submission = {
        JobRequest {
            source_code: req.source_code.clone(),
            language: req.language.clone(),
            user_id: req.user_id,
            contest_id: req.contest_id,
            problem_id: req.problem_id,
        }
    };
    JobResponse {
        id: jobid,
        created_time,
        updated_time,
        submission,
        state: State::Finished,
        result,
        score,
        cases,
        warning: None,
    }
}
/*
    function: to post a JobRequest
    input: body: a web::Json<JobRequest> that bears the JobRequest
    output: Responder
*/
#[post("/jobs")]
pub async fn post_job(body: web::Json<JobRequest>) -> impl Responder {
    let req = body.into_inner();
    let config = &CONFIG.lock().unwrap();
    let userlist = &USER_LIST.lock().unwrap();
    let ctlist = &CONTEST_LIST.lock().unwrap();
    let joblist = &mut JOB_LIST.lock().unwrap();
    let created_time = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut lcheck = false;
    let mut pcheck = false;
    let mut ucheck = false;
    let mut ccheck = false;
    for lang in &config.languages {
        if lang.name == req.language {
            lcheck = true;
            break;
        }
    } // check if the language is in the config
    for pbm in &config.problems {
        if pbm.id == req.problem_id {
            pcheck = true;
            break;
        }
    } // check if the problem id is in the config
    for uindex in 0..userlist.len() {
        if userlist[uindex].id.unwrap() == req.user_id {
            ucheck = true;
            break;
        }
    } // check if the user id is in the userlist
    let mut cexist = false;
    let mut cqulified = false;
    let mut submit = false;
    for cindex in 0..ctlist.len() {
        if ctlist[cindex].id.unwrap() == req.contest_id {
            cexist = true;
            // check if the language is in the ctlist
            let ct = ctlist[cindex].clone();
            if later(&created_time, &ct.from)
                && later(&ct.to, &created_time)
                && ct.user_ids.contains(&req.user_id)
                && ct.problem_ids.contains(&req.problem_id)
            {
                // check if the JobRequest fits the contest's requirement
                cqulified = true;
                let mut count = 0;
                for i in 0..joblist.len() {
                    if joblist[i].submission.user_id == req.user_id
                        && joblist[i].submission.problem_id == req.problem_id
                        && joblist[i].submission.contest_id == req.contest_id
                    {
                        count += 1;
                    }
                }
                // count the times of submission of this contest&&user&&problem in before
                if count < ct.submission_limit {
                    submit = true;
                    ccheck = true; // the overall check related to contest
                }
            }
            break;
        }
    }
    if lcheck && pcheck && ucheck && ccheck {
        // all fits
        let mut js = process_post(&req, &config, joblist.len()).await;
        let conn = &mut MYSQL.lock().unwrap();
        if conn.is_ok() {
            let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
            let stmt = conn
                .prep(
                    "
                INSERT INTO job_submit (id, source_code, language, user_id, contest_id, problem_id) 
                values(?, ?, ?, ?, ?, ?)",
                )
                .unwrap();
            conn.exec_iter(
                stmt,
                (
                    js.id,
                    js.submission.source_code.clone(),
                    js.submission.language.clone(),
                    js.submission.user_id,
                    js.submission.contest_id,
                    js.submission.problem_id,
                ),
            )
            .unwrap();
            // store job_submit
            for i in 0..js.cases.len() {
                let stmt = conn
                    .prep(
                        "
                    INSERT INTO job_cases (jobid, caseid, result, time, memory, info) 
                    values(?, ?, ?, ?, ?, ?)",
                    )
                    .unwrap();
                conn.exec_iter(
                    stmt,
                    (
                        js.id,
                        i,
                        js.cases[i].result.clone().to_string(),
                        js.cases[i].time,
                        js.cases[i].memory,
                        js.cases[i].info.clone(),
                    ),
                )
                .unwrap();
            }
            // store job_cases
            let stmt = conn
                .prep(
                    "
                INSERT INTO joblist (id, create_time, update_time, state, result, score) 
                values(?, ?, ?, ?, ?, ?)",
                )
                .unwrap();
            conn.exec_iter(
                stmt,
                (
                    js.id,
                    js.created_time.clone(),
                    js.updated_time.clone(),
                    js.state.to_string(),
                    js.result.to_string(),
                    js.score,
                ),
            )
            .unwrap();
            // store job itself
        } else {
            js.warning = Some("fail to connect to mysql".to_string());
        }
        joblist.push(js.clone());
        HttpResponse::Ok().json(js)
    } else if !lcheck || !pcheck || !ucheck || !cexist {
        // language or problem id or user id or contest id is not in the config
        HttpResponse::NotFound().json(Error {
            code: 3,
            reason: String::from("ERR_NOT_FOUND"),
            message: String::from("Not Found"),
        })
    } else if !cqulified {
        // not registered in the corresponding contest
        HttpResponse::BadRequest().json(Error {
            code: 1,
            reason: String::from("ERR_INVALID_ARGUMENT"),
            message: String::from("Bad Request"),
        })
    } else if !submit {
        // exceed submit limit
        HttpResponse::BadRequest().json(Error {
            code: 4,
            reason: String::from("ERR_RATE_LIMIT"),
            message: String::from("Bad Request"),
        })
    } else {
        // unexpected err
        HttpResponse::InternalServerError().json(Error {
            code: 6,
            reason: String::from("ERR_INTERNAL"),
            message: String::from("Internal Server Error"),
        })
    }
}
/*
    function: to get the JobResponses according to args
    input: args: a web::Query<JobArgs> that bears the args
    output: Responder
*/
#[get("/jobs")]
pub async fn get_jobs(args: web::Query<JobArgs>) -> impl Responder {
    if (args.contest_id.is_some() && args.contest_id.as_ref().unwrap().parse::<usize>().is_err())
        || (args.problem_id.is_some()
            && args.problem_id.as_ref().unwrap().parse::<usize>().is_err())
        || (args.user_id.is_some() && args.user_id.as_ref().unwrap().parse::<usize>().is_err())
        || (args.from.is_some()
            && NaiveDateTime::parse_from_str(
                &args.from.as_ref().unwrap(),
                "%Y-%m-%dT%H:%M:%S%.3fZ",
            )
            .is_err())
        || (args.to.is_some()
            && NaiveDateTime::parse_from_str(&args.to.as_ref().unwrap(), "%Y-%m-%dT%H:%M:%S%.3fZ")
                .is_err())
        || (args.state.is_some() && string2state(args.state.as_ref().unwrap()).is_err())
        || (args.result.is_some() && string2result(args.result.as_ref().unwrap()).is_err())
    {
        // invalid args
        HttpResponse::NotFound().json(Error {
            code: 1,
            reason: String::from("ERR_INVALID_ARGUMENT"),
            message: String::from("Invalid argument XXX"),
        })
    } else {
        let mut jobs = vec![];
        let joblist = &JOB_LIST.lock().unwrap();
        for i in 0..joblist.len() {
            if joblist[i].jobcheck(&args) {
                jobs.push(joblist[i].clone());
            }
        }
        HttpResponse::Ok().json(jobs)
    }
}
/*
    function: to get the JobResponses according to the jobid
    input: jobid: a web::Path<String> that bears the job id
    output: Responder
*/
#[get("/jobs/{jobid}")]
pub async fn job_id(jobid: web::Path<String>) -> impl Responder {
    let jobid = jobid.parse::<usize>();
    if jobid.is_ok() {
        // if the jobid in the path is a number
        let joblist = &JOB_LIST.lock().unwrap();
        let mut index: Option<usize> = None;
        for i in 0..joblist.len() {
            if joblist[i].id == jobid.clone().unwrap() {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            // found the job
            return HttpResponse::Ok().json(joblist[index.unwrap()].clone());
        }
    }
    HttpResponse::NotFound().json(Error {
        code: 3,
        reason: String::from("ERR_NOT_FOUND"),
        message: String::from("Job 123456 not found."),
    })
}
/*
    function: to retest JobResponses according to the jobid
    input: jobid: a web::Path<String> that bears the job id
    output: Responder
*/
#[put("/jobs/{jobid}")]
pub async fn put_job(jobid: web::Path<String>) -> impl Responder {
    let jobid = jobid.parse::<usize>();
    if jobid.is_ok() {
        let id = jobid.unwrap();
        let joblist = &mut JOB_LIST.lock().unwrap();
        let config = &CONFIG.lock().unwrap();
        let mut index: Option<usize> = None;
        for i in 0..joblist.len() {
            if joblist[i].id == id {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            // found the job
            let index = index.unwrap();
            let before = joblist[index].created_time.clone();
            let req = joblist[index].submission.clone();
            let mut js = process_post(&req, &config, index).await;
            // retest
            js.created_time = before;
            // keep the created time
            let conn = &mut MYSQL.lock().unwrap();
            if conn.is_ok() {
                let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                let stmt = conn.prep("delete from job_cases where jobid = ? ").unwrap();
                conn.exec_iter(stmt, (id,)).unwrap();
                // delete the origin job_cases in mysql
                for i in 0..js.cases.len() {
                    let stmt = conn
                        .prep(
                            "
                        INSERT INTO job_cases (jobid, caseid, result, time, memory, info) 
                        values(?, ?, ?, ?, ?, ?)",
                        )
                        .unwrap();
                    conn.exec_iter(
                        stmt,
                        (
                            js.id,
                            i,
                            js.cases[i].result.clone().to_string(),
                            js.cases[i].time,
                            js.cases[i].memory,
                            js.cases[i].info.clone(),
                        ),
                    )
                    .unwrap();
                }
                // insert with new job_cases
                let stmt = conn.prep("delete from joblist where id = ? ").unwrap();
                conn.exec_iter(stmt, (id,)).unwrap();
                let stmt = conn
                    .prep(
                        "
                    INSERT INTO joblist (id, create_time, update_time, state, result, score) 
                    values(?, ?, ?, ?, ?, ?)",
                    )
                    .unwrap();
                // delete the origin job in joblist in mysql
                conn.exec_iter(
                    stmt,
                    (
                        js.id,
                        js.created_time.clone(),
                        js.updated_time.clone(),
                        js.state.to_string(),
                        js.result.to_string(),
                        js.score,
                    ),
                )
                .unwrap();
                // insert with new job
            } else {
                js.warning = Some("fail to connect to mysql".to_string());
            }
            joblist[index] = js.clone();
            return HttpResponse::Ok().json(js.clone());
        }
    }
    HttpResponse::NotFound().json(Error {
        code: 3,
        reason: String::from("ERR_NOT_FOUND"),
        message: String::from("Job 123456 not found."),
    })
}
/*
    function: to delete JobResponses according to the jobid
    input: jobid: a web::Path<String> that bears the job id
    output: Responder
*/
#[delete("/jobs/{jobid}")]
pub async fn delete_job(jobid: web::Path<String>) -> impl Responder {
    let jobid = jobid.parse::<usize>();
    if jobid.is_ok() {
        let id = jobid.unwrap();
        let joblist = &mut JOB_LIST.lock().unwrap();
        let mut index: Option<usize> = None;
        for i in 0..joblist.len() {
            if joblist[i].id == id {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            // found the job
            let index = index.unwrap();
            if joblist[index].state == State::Queueing {
                joblist.remove(index);
                // delete the job in joblist
                let conn = &mut MYSQL.lock().unwrap();
                if conn.is_ok() {
                    let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                    let stmt = conn.prep("delete from joblist where id = ? ").unwrap();
                    conn.exec_iter(stmt, (id,)).unwrap();
                    let stmt = conn.prep("delete from job_submit where id = ? ").unwrap();
                    conn.exec_iter(stmt, (id,)).unwrap();
                    let stmt = conn.prep("delete from job_cases where jobid = ? ").unwrap();
                    conn.exec_iter(stmt, (id,)).unwrap();
                    // delete the job in mysql
                    return HttpResponse::Ok().json({});
                } else {
                    let warning = "fail to connect to mysql".to_string();
                    return HttpResponse::Ok().json(warning);
                }
            } else {
                // not queueing
                return HttpResponse::BadRequest().json(Error {
                    code: 2,
                    reason: String::from("ERR_INVALID_STATE"),
                    message: String::from("Job 123456 not queuing."),
                });
            }
        }
    }
    HttpResponse::NotFound().json(Error {
        code: 3,
        reason: String::from("ERR_NOT_FOUND"),
        message: String::from("Job 123456 not found."),
    })
}
/*
    function: to post a user
    input: body: a web::Json<User> that bears the user
    output: Responder
*/
#[post("/users")]
pub async fn post_user(body: web::Json<User>) -> impl Responder {
    let userlist = &mut USER_LIST.lock().unwrap();
    let mut user = body.into_inner();
    match user.id {
        Some(id) => {
            let mut unique_id = true;
            let mut unique_name = true;
            let mut index = 0;
            for i in 0..userlist.len() {
                if userlist[i].id.unwrap() == id {
                    unique_id = false;
                    for j in 0..userlist.len() {
                        if j != i && userlist[j].name == user.name {
                            unique_name = false;
                            break;
                        }
                    }
                    index = i;
                    break;
                }
            }
            if !unique_id {
                // id exists
                if unique_name {
                    // name not duplicate, update
                    let conn = &mut MYSQL.lock().unwrap();
                    if conn.is_ok() {
                        let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                        let stmt = conn
                            .prep(
                                "UPDATE userlist
                            SET name = ?
                            where id = ?",
                            )
                            .unwrap();
                        conn.exec_iter(stmt, (user.name.clone(), id)).unwrap();
                        // update the user in mysql
                    } else {
                        user.waring = Some("fail to connect to mysql".to_string());
                    }
                    userlist[index] = user.clone();
                    HttpResponse::Ok().json(user)
                } else {
                    // name duplicate
                    HttpResponse::BadRequest().json(Error {
                        code: 1,
                        reason: String::from("ERR_INVALID_ARGUMENT"),
                        message: String::from("User name 'root' already exists."),
                    })
                }
            } else {
                // id not exist
                HttpResponse::NotFound().json(Error {
                    code: 3,
                    reason: String::from("ERR_NOT_FOUND"),
                    message: String::from("User 123456 not found."),
                })
            }
        }
        None => {
            let mut unique_name = true;
            for i in 0..userlist.len() {
                if userlist[i].name == user.name {
                    unique_name = false;
                    break;
                }
            }
            if unique_name {
                // no id, name not duplicate, new user
                let mut max = userlist[0].id.unwrap();
                for i in 0..userlist.len() {
                    if userlist[i].id.unwrap() > max {
                        max = userlist[i].id.unwrap();
                    }
                }
                let mut newuser = User {
                    id: Some(max + 1),
                    name: user.name,
                    waring: None,
                };
                let ctlist = &mut CONTEST_LIST.lock().unwrap();
                ctlist[0].user_ids.push(max + 1);
                let conn = &mut MYSQL.lock().unwrap();
                if conn.is_ok() {
                    let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                    let stmt = conn
                        .prep(
                            "
                        INSERT INTO userlist (id, name) 
                        values(?, ?)",
                        )
                        .unwrap();
                    conn.exec_iter(stmt, (newuser.id.unwrap(), newuser.name.clone()))
                        .unwrap();
                    let stmt = conn
                        .prep(
                            "
                        INSERT INTO contest_users (id, uid) 
                        values(?, ?)",
                        )
                        .unwrap();
                    conn.exec_iter(stmt, (0, newuser.id.unwrap())).unwrap();
                } else {
                    newuser.waring = Some("fail to connect to mysql".to_string());
                }
                userlist.push(newuser.clone());
                HttpResponse::Ok().json(newuser)
            } else {
                // no id, name exist, invalid args
                HttpResponse::BadRequest().json(Error {
                    code: 1,
                    reason: String::from("ERR_INVALID_ARGUMENT"),
                    message: String::from("User name 'root' already exists."),
                })
            }
        }
    }
}
/*
    function: to get all users
    input: none
    output: Responder
*/
#[get("/users")]
pub async fn get_users() -> impl Responder {
    let userlist = &mut USER_LIST.lock().unwrap();
    for i in 0..userlist.len() {
        let mut min = i;
        for j in i..userlist.len() {
            if userlist[j].id.unwrap() < userlist[min].id.unwrap() {
                min = j;
            }
        }
        userlist.swap(min, i);
    }
    // sort the user list according to id
    HttpResponse::Ok().json(userlist.clone())
}
/*
    function: to check if a contest could be founded
    input: ct: a &Contest of the contest to be checked
    output: true if could, false if couldn't
*/
async fn ctcheck(ct: &Contest) -> bool {
    let config = &CONFIG.lock().unwrap();
    let userlist = &USER_LIST.lock().unwrap();
    let mut pcheck = true;
    let mut ucheck = true;
    for i in 0..ct.problem_ids.len() {
        let mut pcheckie = false;
        for j in 0..config.problems.len() {
            if ct.problem_ids[i] == config.problems[j].id {
                pcheckie = true;
                break;
            }
        }
        if !pcheckie {
            pcheck = false;
            break;
        }
    }
    // check problem ids
    for i in 0..ct.user_ids.len() {
        let mut ucheckie = false;
        for j in 0..userlist.len() {
            if ct.user_ids[i] == userlist[j].id.unwrap() {
                ucheckie = true;
                break;
            }
        }
        if !ucheckie {
            ucheck = false;
            break;
        }
    }
    // check user ids
    pcheck
        && ucheck
        && NaiveDateTime::parse_from_str(&ct.from.clone(), "%Y-%m-%dT%H:%M:%S%.3fZ").is_ok()
        && NaiveDateTime::parse_from_str(&ct.to.clone(), "%Y-%m-%dT%H:%M:%S%.3fZ").is_ok()
}
/*
    function: to post a contest
    input: body: a web::Json<Contest> of the posted contest
    output: Responder
*/
#[post("/contests")]
pub async fn post_contest(body: web::Json<Contest>) -> impl Responder {
    let mut contest = body.into_inner();
    let ctlist = &mut CONTEST_LIST.lock().unwrap();
    match contest.id {
        Some(id) => {
            let mut idcheck = false;
            let mut index = 0;
            for i in 0..ctlist.len() {
                if ctlist[i].id.unwrap() == id {
                    idcheck = true;
                    index = i;
                    break;
                }
            }
            if id != 0 && idcheck && ctcheck(&contest).await {
                // id exist and not 0, contest passed the check, update
                let conn = &mut MYSQL.lock().unwrap();
                if conn.is_ok() {
                    let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                    let stmt = conn
                        .prep(
                            "UPDATE contest_list
                        SET name = ?, fromtime = ?, totime = ?, submission_limit = ?
                        where id = ?",
                        )
                        .unwrap();
                    conn.exec_iter(
                        stmt,
                        (
                            contest.name.clone(),
                            contest.from.clone(),
                            contest.to.clone(),
                            contest.submission_limit,
                            contest.id,
                        ),
                    )
                    .unwrap();
                    // update contest_list in mysql
                    let stmt = conn
                        .prep("delete from contest_problems where id = ? ")
                        .unwrap();
                    conn.exec_iter(stmt, (id,)).unwrap();
                    // delete origin contest_problems in mysql
                    for i in 0..contest.problem_ids.len() {
                        let stmt = conn
                            .prep(
                                "
                            INSERT INTO contest_problems (id, pid) 
                            values(?, ?)",
                            )
                            .unwrap();
                        conn.exec_iter(stmt, (contest.id, contest.problem_ids[i]))
                            .unwrap();
                    }
                    // insert with new problem ids
                    let stmt = conn
                        .prep("delete from contest_users where id = ? ")
                        .unwrap();
                    conn.exec_iter(stmt, (id,)).unwrap();
                    // delete origin contest_users in mysql
                    for i in 0..contest.user_ids.len() {
                        let stmt = conn
                            .prep(
                                "
                            INSERT INTO contest_users (id, uid) 
                            values(?, ?)",
                            )
                            .unwrap();
                        conn.exec_iter(stmt, (contest.id, contest.user_ids[i]))
                            .unwrap();
                    }
                    // insert with new users
                } else {
                    contest.waring = Some("fail to connect to mysql".to_string());
                }
                ctlist[index] = contest.clone();
                HttpResponse::Ok().json(contest)
            } else {
                HttpResponse::NotFound().json(Error {
                    code: 3,
                    reason: String::from("ERR_NOT_FOUND"),
                    message: String::from("Contest 114514 not found."),
                })
            }
        }
        None => {
            if ctcheck(&contest).await {
                // no id, pass the check, new contest
                let mut max = ctlist[0].id.unwrap();
                for i in 0..ctlist.len() {
                    if ctlist[i].id.unwrap() > max {
                        max = ctlist[i].id.unwrap();
                    }
                }
                contest.id = Some(max + 1);
                // id = maxid + 1
                let conn = &mut MYSQL.lock().unwrap();
                if conn.is_ok() {
                    let mut conn = conn.as_ref().unwrap().get_conn().unwrap();
                    let stmt = conn
                        .prep(
                            "
                        INSERT INTO contest_list (id, name, fromtime, totime, submission_limit) 
                        values(?, ?, ?, ?, ?)",
                        )
                        .unwrap();
                    conn.exec_iter(
                        stmt,
                        (
                            contest.id,
                            contest.name.clone(),
                            contest.from.clone(),
                            contest.to.clone(),
                            contest.submission_limit,
                        ),
                    )
                    .unwrap();
                    // insert new contest in contest_list in mysql
                    for i in 0..contest.problem_ids.len() {
                        let stmt = conn
                            .prep(
                                "
                            INSERT INTO contest_problems (id, pid) 
                            values(?, ?)",
                            )
                            .unwrap();
                        conn.exec_iter(stmt, (contest.id, contest.problem_ids[i]))
                            .unwrap();
                    }
                    // insert new problems in contest_problems in mysql
                    for i in 0..contest.user_ids.len() {
                        let stmt = conn
                            .prep(
                                "
                            INSERT INTO contest_users (id, uid) 
                            values(?, ?)",
                            )
                            .unwrap();
                        conn.exec_iter(stmt, (contest.id, contest.user_ids[i]))
                            .unwrap();
                    }
                    // insert new users in contest_users in mysql
                } else {
                    contest.waring = Some("fail to connect to mysql".to_string());
                }
                ctlist.push(contest.clone());
                HttpResponse::Ok().json(contest)
            } else {
                HttpResponse::NotFound().json(Error {
                    code: 3,
                    reason: String::from("ERR_NOT_FOUND"),
                    message: String::from("Contest 114514 not found."),
                })
            }
        }
    }
}
/*
    function: to get all contests
    input: none
    output: Responder
*/
#[get("/contests")]
pub async fn get_contests() -> impl Responder {
    let ctlist = &mut CONTEST_LIST.lock().unwrap();
    let mut result = vec![];
    if ctlist.len() != 1 {
        for i in 1..ctlist.len() {
            let mut min = i;
            for j in i..ctlist.len() {
                if ctlist[j].id.unwrap() < ctlist[min].id.unwrap() {
                    min = j
                }
            }
            ctlist.swap(min, i);
        }
        for i in 1..ctlist.len() {
            result.push(ctlist[i].clone());
        }
    }
    // skip the contest0
    HttpResponse::Ok().json(result.clone())
}
/*
    function: to get a contest according to its id
    input: cid: a web::Path<String> that bears the id of the contest
    output: Responder
*/
#[get("/contests/{cid}")]
pub async fn contest_id(cid: web::Path<String>) -> impl Responder {
    let cid = cid.parse::<usize>();
    if cid.is_ok() {
        let id = cid.unwrap();
        let ctlist = &CONTEST_LIST.lock().unwrap();
        let mut index: Option<usize> = None;
        for i in 0..ctlist.len() {
            if ctlist[i].id.unwrap() == id {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            return HttpResponse::Ok().json(ctlist[index.unwrap()].clone());
        }
    }
    HttpResponse::NotFound().json(Error {
        code: 3,
        reason: String::from("ERR_NOT_FOUND"),
        message: String::from("Contest 114514 not found."),
    })
}
/*
    function: to get a contest ranklist according to the id and the args
    input: cid: a web::Path<String> that bears the id of the contest
           args: a web::Query<ContestArgs> that bears the ranking rules
    output: Responder
*/
#[get("/contests/{cid}/ranklist")]
pub async fn ranklist(cid: web::Path<String>, args: web::Query<ContestArgs>) -> impl Responder {
    let args = args.into_inner();
    let cid = cid.parse::<usize>();
    let mut contest = None;
    if cid.is_ok() {
        let id = cid.unwrap();
        let ctlist = &CONTEST_LIST.lock().unwrap();
        let mut index: Option<usize> = None;
        for i in 0..ctlist.len() {
            if ctlist[i].id.unwrap() == id {
                index = Some(i);
                break;
            }
        }
        if index.is_some() {
            contest = Some(ctlist[index.unwrap()].clone());
        }
    }
    if contest.is_none() {
        // can't find the contest of the given id
        return HttpResponse::NotFound().json(Error {
            code: 3,
            reason: String::from("ERR_NOT_FOUND"),
            message: String::from("Contest 114514 not found."),
        });
    }
    HttpResponse::Ok().json(contest.unwrap().contest_ranker(&args).await)
}
