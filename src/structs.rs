use std::usize;

use crate::config::ProblemType;

use super::{CONFIG, JOB_LIST, USER_LIST};
use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use strum::{self, Display};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    pub source_code: String,
    pub language: String,
    pub user_id: usize,
    pub contest_id: usize,
    pub problem_id: usize,
}

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled,
}
/*
function: to transform a String into State
input: state: a &str to be transformed
output: the corresponding State or Err
*/
pub fn string2state(state: &str) -> Result<State, String> {
    match state {
        "Queueing" => Ok(State::Queueing),
        "Running" => Ok(State::Running),
        "Finished" => Ok(State::Finished),
        "Canceled" => Ok(State::Canceled),
        _ => Err("Can't transform into State".to_string()),
    }
}

#[derive(Clone, Debug, Display, PartialEq, Serialize, Deserialize)]
pub enum MyResult {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    #[strum(serialize = "Compilation Error")]
    CE,
    #[serde(rename = "Compilation Success")]
    #[strum(serialize = "Compilation Success")]
    CS,
    #[serde(rename = "Wrong Answer")]
    #[strum(serialize = "Wrong Answer")]
    WA,
    #[serde(rename = "Runtime Error")]
    #[strum(serialize = "Runtime Error")]
    RE,
    #[serde(rename = "Time Limit Exceeded")]
    #[strum(serialize = "Time Limit Exceeded")]
    TLE,
    #[serde(rename = "Memory Limit Exceeded")]
    #[strum(serialize = "Memory Limit Exceeded")]
    MLE,
    #[serde(rename = "System Error")]
    #[strum(serialize = "System Error")]
    SE,
    #[serde(rename = "SPJ Error")]
    #[strum(serialize = "SPJ Error")]
    SPJE,
    Skipped,
}
/*
function: to transform a String into Myresult
input: myresult: a &str to be transformed
output: the corresponding Myresult or Err
*/
pub fn string2result(myresult: &str) -> Result<MyResult, String> {
    match myresult {
        "Waiting" => Ok(MyResult::Waiting),
        "Running" => Ok(MyResult::Running),
        "Accepted" => Ok(MyResult::Accepted),
        "Compilation Error" => Ok(MyResult::CE),
        "Compilation Success" => Ok(MyResult::CS),
        "Wrong Answer" => Ok(MyResult::WA),
        "Runtime Error" => Ok(MyResult::RE),
        "Time Limit Exceeded" => Ok(MyResult::TLE),
        "Memory Limit Exceeded" => Ok(MyResult::MLE),
        "System Error" => Ok(MyResult::SE),
        "SPJ Error" => Ok(MyResult::SPJE),
        "Skipped" => Ok(MyResult::Skipped),
        _ => Err("Can't transform into MyResult".to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: usize,
    pub result: MyResult,
    pub time: u128,
    pub memory: f64,
    pub info: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub id: usize,
    pub created_time: String,
    pub updated_time: String,
    pub submission: JobRequest,
    pub state: State,
    pub result: MyResult,
    pub score: f64,
    pub cases: Vec<Case>,
    pub warning: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct JobArgs {
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub contest_id: Option<String>,
    pub problem_id: Option<String>,
    pub language: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub state: Option<String>,
    pub result: Option<String>,
}
/*
function: to compare whether the first time is later than the second
input: mytime, yourtime two time as &str in %Y-%m-%dT%H:%M:%S%.3fZ form to be compared
output: if mytime is later than your time, true, otherwise, false
*/
pub fn later(mytime: &str, yourtime: &str) -> bool {
    let mytime = NaiveDateTime::parse_from_str(mytime, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
    let yourtime = NaiveDateTime::parse_from_str(yourtime, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
    if mytime > yourtime {
        true
    } else {
        false
    }
}

impl JobResponse {
    /*
    function: to check if the problem id is as requested
    input: pid: an option<String> of the problem id to be compared
    output: true if the problem id is as requested or when pid is None, otherwise false
    */
    pub fn pmatch(&self, pid: &Option<String>) -> bool {
        if pid.is_none() {
            true // which means problem id isn't listed in the request for getting a job
        } else {
            if self.submission.problem_id == pid.clone().unwrap().parse::<usize>().unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the contest id is as requested
    input: cid: an option<String> of the contest id to be compared
    output: true if the contest id is as requested or when cid is None, otherwise false
    */
    pub fn cmatch(&self, cid: &Option<String>) -> bool {
        if cid.is_none() {
            true // which means problem id isn't listed in the request for getting a job
        } else {
            if self.submission.contest_id == cid.clone().unwrap().parse::<usize>().unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the user id is as requested
    input: uid: an option<String> of the user id to be compared
    output: true if the user id is as requested or when uid is None, otherwise false
    */
    pub fn umatch(&self, uid: &Option<String>) -> bool {
        if uid.is_none() {
            true // which means problem id isn't listed in the request for getting a job
        } else {
            if self.submission.user_id == uid.clone().unwrap().parse::<usize>().unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the user name is as requested
    input: uname: an option<String> of the user name to be compared
    output: true if the user name is as requested or when uname is None, otherwise false
    */
    pub fn unmatch(&self, uname: &Option<String>) -> bool {
        if uname.is_none() {
            true // which means problem id isn't listed in the request for getting a job
        } else {
            if name(self.submission.user_id) == uname.clone().unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the language is as requested
    input: lang: an option<String> of the language to be compared
    output: true if the language is as requested or when lang is None, otherwise false
    */
    pub fn lmatch(&self, lang: &Option<String>) -> bool {
        if lang.is_none() {
            true // which means language isn't listed in the request for getting a job
        } else {
            if self.submission.language == lang.clone().unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the created time is as from requested
    input: from: an option<String> of the time string to be compared
    output: true if the created_time is later than from or when from is None, otherwise false
    */
    pub fn fmatch(&self, from: &Option<String>) -> bool {
        if from.is_none() {
            true // which means from isn't listed in the request for getting a job
        } else {
            if later(&self.created_time, &from.as_ref().unwrap()) {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the created time is as to requested
    input: to: an option<String> of the time string to be compared
    output: true if to is later than the created_time or when to is None, otherwise false
    */
    pub fn tmatch(&self, to: &Option<String>) -> bool {
        if to.is_none() {
            true // which means to isn't listed in the request for getting a job
        } else {
            if later(&self.created_time, &to.as_ref().unwrap()) {
                false
            } else {
                true
            }
        }
    }
    /*
    function: to check if the state is as requested
    input: state: an option<String> of the state to be compared
    output: true if the state is as requested or when state is None, otherwise false
    */
    pub fn smatch(&self, state: &Option<String>) -> bool {
        if state.is_none() {
            true // which means state isn't listed in the request for getting a job
        } else {
            if self.state == string2state(&state.clone().unwrap()).unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the result is as requested
    input: result: an option<MyResult> of the result to be compared
    output: true if the result is as requested or when result is None, otherwise false
    */
    pub fn rmatch(&self, result: &Option<String>) -> bool {
        if result.is_none() {
            true // which means result isn't listed in the request for getting a job
        } else {
            if self.result == string2result(&result.clone().unwrap()).unwrap() {
                true
            } else {
                false
            }
        }
    }
    /*
    function: to check if the JobResponse is as requested
    input: args: the args listed when getting a job
    output: true if the JobResponse is as requested, otherwise false
    */
    pub fn jobcheck(&self, args: &JobArgs) -> bool {
        self.pmatch(&args.problem_id)
            && self.cmatch(&args.contest_id)
            && self.umatch(&args.user_id)
            && self.lmatch(&args.language)
            && self.fmatch(&args.from)
            && self.tmatch(&args.to)
            && self.smatch(&args.state)
            && self.rmatch(&args.result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<usize>,
    pub name: String,
    pub waring: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contest {
    pub id: Option<usize>,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<usize>,
    pub user_ids: Vec<usize>,
    pub submission_limit: usize,
    pub waring: Option<String>,
}

#[derive(Debug, Display, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Srule {
    #[strum(serialize = "latest")]
    Latest,
    #[strum(serialize = "highest")]
    Highest,
}

#[derive(Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tiebreaker {
    #[strum(serialize = "submission_time")]
    SubmissionTime,
    #[strum(serialize = "submission_count")]
    SubmissionCount,
    #[strum(serialize = "user_id")]
    UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContestArgs {
    pub scoring_rule: Option<Srule>,
    pub tie_breaker: Option<Tiebreaker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRank {
    pub user: User,
    pub rank: usize,
    pub scores: Vec<f64>,
}
/*
    function: to select all the jobs that match the user id and the problem id
    input: uid: a usize of the user id to be matched
           pid: a usize of the problem id to be matched
    output: a vector of JobResponse that match the user id and the problem id
*/
pub async fn candidates(uid: usize, pid: usize) -> Vec<JobResponse> {
    let joblist = &JOB_LIST.lock().unwrap();
    let mut candidates = vec![];
    for i in 0..joblist.len() {
        if joblist[i].submission.user_id == uid && joblist[i].submission.problem_id == pid {
            candidates.push(joblist[i].clone());
        }
    }
    candidates
}
/*
    function: to select the latest JobResponse from candidates
    input: candidates: a &vec of JobResponse to be selected
    output: a (String, f64) each stands for the latest JobResponse's created_time and score
*/
pub async fn latest(candidates: &Vec<JobResponse>) -> (String, f64) {
    if candidates.is_empty() {
        (String::from("9999-12-31T23:59:59.999Z"), 0.0)
    } else {
        let mut result = candidates[0].clone();
        for i in 0..candidates.len() {
            if later(&candidates[i].created_time, &result.created_time) {
                result = candidates[i].clone();
            }
        }
        (result.created_time, result.score)
    }
}
/*
    function: to select the highest JobResponse from candidates
    input: candidates: a &vec of JobResponse to be selected
    output: a (String, f64) each stands for the highest JobResponse's created_time and score
*/
pub async fn highest(candidates: &Vec<JobResponse>) -> (String, f64) {
    if candidates.is_empty() {
        (String::from("9999-12-31T23:59:59.999Z"), 0.0)
    } else {
        let mut result = candidates[0].score;
        let mut scorer = vec![];
        for i in 0..candidates.len() {
            if candidates[i].score > result {
                result = candidates[i].score;
            }
        } // result is the highest score in the candidates
        for i in 0..candidates.len() {
            if candidates[i].score == result {
                scorer.push(candidates[i].clone());
            }
        } // scorer is a vector of JobResponse that all score the highest
        let mut result = scorer[0].clone();
        for i in 0..scorer.len() {
            if later(&result.created_time, &scorer[i].created_time) {
                result = scorer[i].clone();
            }
        } // result is the earliest JobResponse in the scorer
        (result.created_time, result.score)
    }
}
/*
    function: to find the name of the user according to the id
    input: uid: a usize of the id of the user to be find
    output: a String of the user's name or an empty String
*/
pub fn name(uid: usize) -> String {
    let userlist = &mut USER_LIST.lock().unwrap();
    for i in 0..userlist.len() {
        if userlist[i].id.unwrap() == uid {
            return userlist[i].name.clone();
        }
    }
    String::from("")
}
/*
    function: to break the tie between the candidates according to submission time
    input: candidates: a vec of ordered vecs, each vec stores a user information with the same total score,
                       each information with the form (uszie, f64, Vev<f64>, String, uszie),
                       where each stands for: user_id, total score, scores of each problems, submission time, the times of submission
    output: a vec of UserRank which is the so-called ranklist
*/
pub async fn submission_time(
    mut candidates: Vec<Vec<(usize, f64, Vec<f64>, String, usize)>>,
) -> Vec<UserRank> {
    let mut before = 0;
    let mut results = vec![];
    for i in 0..candidates.len() {
        for j in 0..candidates[i].len() {
            let mut earliest = j;
            for k in j..candidates[i].len() {
                if later(&candidates[i][earliest].3, &candidates[i][k].3) {
                    earliest = k;
                }
            }
            candidates[i].swap(earliest, j);
        } // sort the same total score users according to submission time
        for j in 0..candidates[i].len() {
            results.push(UserRank {
                user: User {
                    id: Some(candidates[i][j].0),
                    name: name(candidates[i][j].0),
                    waring: None,
                },
                rank: before + j + 1,
                scores: candidates[i][j].2.clone(),
            })
        } // create the ranklist
        before += candidates[i].len(); // before stands for the number of users with higher total score
    }
    results
}
/*
    function: to break the tie between the candidates according to submission count
    input: candidates: a vec of ordered vecs, each vec stores user information with the same total score,
                       each information with the form (uszie, f64, Vev<f64>, String, uszie),
                       where each stands for: user_id, total score, scores of each problems, submission time, submission count
    output: a vec of UserRank which is the so-called ranklist
*/
pub async fn submission_count(
    mut candidates: Vec<Vec<(usize, f64, Vec<f64>, String, usize)>>,
) -> Vec<UserRank> {
    let mut before = 0;
    let mut results = vec![];
    for i in 0..candidates.len() {
        for j in 0..candidates[i].len() {
            let mut least = j;
            for k in j..candidates[i].len() {
                if &candidates[i][least].4 > &candidates[i][k].4 {
                    least = k;
                }
            }
            candidates[i].swap(least, j);
        } // sort the same total score users according to submission count
        let mut count = 1;
        let mut same_submits = vec![];
        for j in 0..candidates[i].len() - 1 {
            if candidates[i][j].4 == candidates[i][j + 1].4 {
                count += 1;
                if i == candidates[i].len() - 2 {
                    // when the second to last is of the same submission count of the last
                    let mut same_submit = vec![];
                    for k in i - (count - 2)..=i + 1 {
                        same_submit.push(candidates[i][k].clone());
                    }
                    for k in 0..same_submit.len() {
                        let mut id = k;
                        for l in k..same_submit.len() {
                            if same_submit[l].0 < same_submit[id].0 {
                                id = l;
                            }
                        }
                        same_submit.swap(id, k);
                    } // sort same_submit according to user_id
                    same_submits.push(same_submit);
                }
            } else {
                // when the next candidate id not of same submission count as this one
                let mut same_submit = vec![];
                for k in j - (count - 1)..=j {
                    same_submit.push(candidates[i][k].clone());
                }
                for k in 0..same_submit.len() {
                    let mut id = k;
                    for l in k..same_submit.len() {
                        if same_submit[l].0 < same_submit[id].0 {
                            id = l;
                        }
                    }
                    same_submit.swap(id, k);
                } // sort same_submit according to user_id
                same_submits.push(same_submit);
                count = 1;
            }
        }
        if count == 1 {
            //when the last candidate is of unique submission count
            same_submits.push(vec![candidates[i].last().unwrap().clone()]);
        }
        // same_submits is a vec of vecs transformed from the vec of user information with the same total score,
        // where each vec stores the user information with the same total score and the same submission count
        let mut beforee = 0;
        for j in 0..same_submits.len() {
            for k in 0..same_submits[j].len() {
                results.push(UserRank {
                    user: User {
                        id: Some(same_submits[j][k].0),
                        name: name(same_submits[j][k].0),
                        waring: None,
                    },
                    rank: before + beforee + 1,
                    scores: same_submits[j][k].2.clone(),
                })
            }
            beforee += same_submits[j].len(); // beforee stands for the number of users with same total score but less submission count
        }
        before += candidates[i].len(); // before stand for the number of users with higher total score
    }
    results
}
/*
    function: to break the tie between the candidates according to user id
    input: candidates: a vec of ordered vecs, each vec stores user information with the same total score,
                       each information with the form (uszie, f64, Vev<f64>, String, uszie),
                       where each stands for: user_id, total score, scores of each problems, submission time, submission count
    output: a vec of UserRank which is the so-called ranklist
*/
pub async fn user_id(
    mut candidates: Vec<Vec<(usize, f64, Vec<f64>, String, usize)>>,
) -> Vec<UserRank> {
    let mut before = 0;
    let mut results = vec![];
    for i in 0..candidates.len() {
        for j in 0..candidates[i].len() {
            let mut idmin = j;
            for k in j..candidates[i].len() {
                if &candidates[i][idmin].0 > &candidates[i][k].0 {
                    idmin = k;
                }
            }
            candidates[i].swap(idmin, j);
        } // sort the same total score users according to user id
        for j in 0..candidates[i].len() {
            results.push(UserRank {
                user: User {
                    id: Some(candidates[i][j].0),
                    name: name(candidates[i][j].0),
                    waring: None,
                },
                rank: before + j + 1,
                scores: candidates[i][j].2.clone(),
            })
        } // create the ranklist
        before += candidates[i].len(); // before stands for the number of users with higher total score
    }
    results
}
/*
    function: to break the tie between the candidates when there is no explicit tie-breaker
    input: candidates: a vec of ordered vecs, each vec stores user information with the same total score,
                       each information with the form (uszie, f64, Vev<f64>, String, uszie),
                       where each stands for: user_id, total score, scores of each problems, submission time, submission count
    output: a vec of UserRank which is the so-called ranklist
*/
pub async fn none(
    mut candidates: Vec<Vec<(usize, f64, Vec<f64>, String, usize)>>,
) -> Vec<UserRank> {
    let mut before = 0;
    let mut results = vec![];
    for i in 0..candidates.len() {
        for j in 0..candidates[i].len() {
            let mut idmin = j;
            for k in j..candidates[i].len() {
                if &candidates[i][j].0 > &candidates[i][k].0 {
                    idmin = k;
                }
            }
            candidates[i].swap(idmin, j);
        } // sort the same total score users according to user id
        for j in 0..candidates[i].len() {
            results.push(UserRank {
                user: User {
                    id: Some(candidates[i][j].0),
                    name: name(candidates[i][j].0),
                    waring: None,
                },
                rank: before + 1,
                scores: candidates[i][j].2.clone(),
            })
        } // create the ranklist
        before += candidates[i].len(); // before stands for the number of users with higher total score
    }
    results
}

impl Contest {
    pub async fn contest_ranker(&self, args: &ContestArgs) -> Vec<UserRank> {
        let mut results = vec![];
        let config = CONFIG.lock().unwrap();
        for i in 0..self.user_ids.len() {
            let mut times = vec![];
            let mut total_score = 0.0;
            let mut scores = vec![];
            let mut submission_count = 0;
            for j in 0..self.problem_ids.len() {
                // for each user, address each of their problems
                let possible = candidates(self.user_ids[i], self.problem_ids[j]).await;
                // find all possible JobResponses that match the problem id and the user id
                submission_count += possible.len();
                let pid = self.problem_ids[j];
                let mut csub = 0;
                for k in 0..config.problems.len() {
                    if config.problems[k].id == pid {
                        csub = k;
                    }
                }
                // csub is the problem's order in the Config
                if config.problems[csub].ty == ProblemType::DynamicRanking {
                    // dynamic rankink
                    let mut selected_possible = vec![];
                    for i in 0..possible.len() {
                        if possible[i].result == MyResult::Accepted {
                            selected_possible.push(possible[i].clone());
                        }
                    }
                    // select accepted JobResponse from the possible
                    if !selected_possible.is_empty() {
                        // when there exist accepted JobResponses, ignore the scoring rule
                        let drr = config.problems[csub]
                            .misc
                            .as_ref()
                            .unwrap()
                            .dynamic_ranking_ratio
                            .unwrap();
                        // drr is the rate of competition score
                        let mut latest = selected_possible[0].clone();
                        for l in 0..selected_possible.len() {
                            if later(&selected_possible[l].created_time, &latest.created_time) {
                                latest = selected_possible[l].clone();
                            }
                        }
                        // find the latest accepted JobResponse
                        let joblist = JOB_LIST.lock().unwrap();
                        let mut score = 0.0;
                        for l in 1..latest.cases.len() {
                            let mut min = latest.cases[l].time;
                            for m in 0..joblist.len() {
                                if joblist[m].submission.problem_id == pid
                                    && joblist[m].submission.contest_id == self.id.unwrap()
                                    && joblist[m].result == MyResult::Accepted
                                    && joblist[m].cases[l].time < min
                                {
                                    min = joblist[m].cases[l].time;
                                }
                            }
                            // find the least time of this problem in all the JobResponses
                            // that was accepted, and in this contest
                            score += config.problems[csub].cases[l - 1].score
                                * (1.0 - drr + drr * min as f64 / latest.cases[l].time as f64);
                            // calculate the score
                        }
                        times.push(latest.created_time);
                        total_score += score;
                        scores.push(score);
                    }
                } else {
                    // not dynamic ranking or no accepted JobResponse under dynamic ranking mode,
                    // the score is the same as in JobResponse and should be selected according to scoring rule
                    if args.scoring_rule == Some(Srule::Highest) {
                        let high = highest(&possible.clone()).await;
                        times.push(high.0);
                        total_score += high.1;
                        scores.push(high.1);
                    } else {
                        let late = latest(&possible.clone()).await;
                        times.push(late.0);
                        total_score += late.1;
                        scores.push(late.1);
                    }
                }
            }
            let mut submission_time = String::from("0000-01-01T00:00:00.001Z");
            for j in 0..times.len() {
                if times[j] == String::from("9999-12-31T23:59:59.999Z") {
                    continue;
                } else {
                    if later(&times[j], &submission_time) {
                        submission_time = times[j].clone();
                    }
                }
            }
            // find the latest submission time in all problems
            if submission_time == String::from("0000-01-01T00:00:00.001Z") {
                // when this user had no submission at all
                submission_time = String::from("9999-12-31T23:59:59.999Z");
            }
            results.push((
                self.user_ids[i],
                total_score,
                scores,
                submission_time,
                submission_count,
            ));
        }
        for i in 0..results.len() {
            let mut max = i;
            for j in i..results.len() {
                if results[j].1 > results[max].1 {
                    max = j;
                }
            }
            results.swap(i, max);
        }
        // sort the results according to total score
        let mut count = 1;
        let mut same_scores = vec![];
        for i in 0..results.len() - 1 {
            if results[i].1 == results[i + 1].1 {
                // when the next's total score is the same as this one's
                count += 1;
                if i == results.len() - 2 {
                    // when the second to last's total score is the same as the last one's
                    let mut same_score = vec![];
                    for j in i - (count - 2)..=i + 1 {
                        same_score.push(results[j].clone());
                    }
                    same_scores.push(same_score);
                }
            } else {
                // when the next's total score is different from this one's
                let mut same_score = vec![];
                for j in (i - (count - 1))..=i {
                    same_score.push(results[j].clone());
                }
                same_scores.push(same_score);
                count = 1;
            }
        }
        if count == 1 {
            // when the last one's total score is unique
            same_scores.push(vec![results.last().unwrap().clone()]);
        }
        match args.tie_breaker {
            // break the tie, return with the ranklist
            Some(Tiebreaker::SubmissionTime) => submission_time(same_scores).await,
            Some(Tiebreaker::SubmissionCount) => submission_count(same_scores).await,
            Some(Tiebreaker::UserId) => user_id(same_scores).await,
            None => none(same_scores).await,
        }
    }
}
