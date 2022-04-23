use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use darkfi::util::serial::{Decodable, Encodable, SerialDecodable, SerialEncodable};

use darkfi::util::serial::VarInt;

use crate::{
    error::{TaudError, TaudResult},
    tasks::Tasks,
    util::{find_free_id, get_current_time, random_ref_id, Timestamp},
};

#[derive(Clone, Debug, Serialize, Deserialize, SerialEncodable, SerialDecodable, PartialEq)]
struct TaskEvent {
    action: String,
    timestamp: Timestamp,
}

impl TaskEvent {
    fn new(action: String) -> Self {
        Self { action, timestamp: get_current_time() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SerialDecodable, SerialEncodable, PartialEq)]
pub struct Comment {
    content: String,
    author: String,
    timestamp: Timestamp,
}

impl Comment {
    pub fn new(content: &str, author: &str) -> Self {
        Self { content: content.into(), author: author.into(), timestamp: get_current_time() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskEvents(Vec<TaskEvent>);
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskComments(Vec<Comment>);
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskProjects(Vec<String>);
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskAssigns(Vec<String>);

#[derive(Clone, Debug, Serialize, Deserialize, SerialEncodable, SerialDecodable, PartialEq)]
pub struct TaskInfo {
    pub(crate) ref_id: String,
    id: u32,
    title: String,
    desc: String,
    assign: TaskAssigns,
    project: TaskProjects,
    due: Option<Timestamp>,
    rank: f32,
    created_at: Timestamp,
    events: TaskEvents,
    comments: TaskComments,
}

impl TaskInfo {
    pub fn new(
        title: &str,
        desc: &str,
        due: Option<Timestamp>,
        rank: f32,
        dataset_path: &Path,
    ) -> TaudResult<Self> {
        // generate ref_id
        let ref_id = random_ref_id();

        let created_at: Timestamp = get_current_time();

        let task_ids: Vec<u32> =
            Tasks::load_current_open_tasks(dataset_path)?.into_iter().map(|t| t.id).collect();

        let id: u32 = find_free_id(&task_ids);

        if let Some(d) = &due {
            if *d < get_current_time() {
                return Err(TaudError::InvalidDueTime)
            }
        }

        Ok(Self {
            ref_id,
            id,
            title: title.into(),
            desc: desc.into(),
            assign: TaskAssigns(vec![]),
            project: TaskProjects(vec![]),
            due,
            rank,
            created_at,
            comments: TaskComments(vec![]),
            events: TaskEvents(vec![]),
        })
    }

    pub fn load(ref_id: &str, dataset_path: &Path) -> TaudResult<Self> {
        let task = crate::util::load::<Self>(&Self::get_path(ref_id, dataset_path))?;
        Ok(task)
    }

    pub fn save(&self, dataset_path: &Path) -> TaudResult<()> {
        crate::util::save::<Self>(&Self::get_path(&self.ref_id, dataset_path), self)
            .map_err(TaudError::Darkfi)?;

        if self.get_state() == "stop" {
            self.deactivate(dataset_path)?;
        } else {
            self.activate(dataset_path)?;
        }

        Ok(())
    }

    pub fn activate(&self, path: &Path) -> TaudResult<()> {
        let mut mt = Tasks::load_or_create(path, "pending")?;
        mt.add(&self.ref_id);
        mt.save(path, "pending")
    }

    pub fn deactivate(&self, path: &Path) -> TaudResult<()> {
        let mut mt = Tasks::load_or_create(path, "pending")?;
        let mut temp_mt = Tasks::load_or_create(path, "completed")?;
        mt.remove(&self.ref_id);
        mt.save(path, "pending")?;
        temp_mt.add(&self.ref_id);
        temp_mt.save(path, "completed")
    }

    pub fn get_state(&self) -> String {
        if let Some(ev) = self.events.0.last() {
            ev.action.clone()
        } else {
            "open".into()
        }
    }

    fn get_path(ref_id: &str, dataset_path: &Path) -> PathBuf {
        dataset_path.join("task").join(ref_id)
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }

    pub fn set_desc(&mut self, desc: &str) {
        self.desc = desc.into();
    }

    pub fn set_assign(&mut self, assign: &[String]) {
        self.assign = TaskAssigns(assign.to_owned());
    }

    pub fn set_project(&mut self, project: &[String]) {
        self.project = TaskProjects(project.to_owned());
    }

    pub fn set_comment(&mut self, c: Comment) {
        self.comments.0.push(c);
    }

    pub fn set_rank(&mut self, r: f32) {
        self.rank = r;
    }

    pub fn set_due(&mut self, d: Option<Timestamp>) {
        self.due = d;
    }

    pub fn set_state(&mut self, action: &str) {
        if self.get_state() == action {
            return
        }
        self.events.0.push(TaskEvent::new(action.into()));
    }
}

impl Encodable for TaskEvents {
    fn encode<S: io::Write>(&self, s: S) -> darkfi::Result<usize> {
        encode_vec(&self.0, s)
    }
}

impl Decodable for TaskEvents {
    fn decode<D: io::Read>(d: D) -> darkfi::Result<Self> {
        Ok(Self(decode_vec(d)?))
    }
}
impl Encodable for TaskComments {
    fn encode<S: io::Write>(&self, s: S) -> darkfi::Result<usize> {
        encode_vec(&self.0, s)
    }
}

impl Decodable for TaskComments {
    fn decode<D: io::Read>(d: D) -> darkfi::Result<Self> {
        Ok(Self(decode_vec(d)?))
    }
}
impl Encodable for TaskProjects {
    fn encode<S: io::Write>(&self, s: S) -> darkfi::Result<usize> {
        encode_vec(&self.0, s)
    }
}

impl Decodable for TaskProjects {
    fn decode<D: io::Read>(d: D) -> darkfi::Result<Self> {
        Ok(Self(decode_vec(d)?))
    }
}

impl Encodable for TaskAssigns {
    fn encode<S: io::Write>(&self, s: S) -> darkfi::Result<usize> {
        encode_vec(&self.0, s)
    }
}

impl Decodable for TaskAssigns {
    fn decode<D: io::Read>(d: D) -> darkfi::Result<Self> {
        Ok(Self(decode_vec(d)?))
    }
}

fn encode_vec<T: Encodable, S: io::Write>(vec: &[T], mut s: S) -> darkfi::Result<usize> {
    let mut len = 0;
    len += VarInt(vec.len() as u64).encode(&mut s)?;
    for c in vec.iter() {
        len += c.encode(&mut s)?;
    }
    Ok(len)
}

fn decode_vec<T: Decodable, D: io::Read>(mut d: D) -> darkfi::Result<Vec<T>> {
    let len = VarInt::decode(&mut d)?.0;
    let mut ret = Vec::with_capacity(len as usize);
    for _ in 0..len {
        ret.push(Decodable::decode(&mut d)?);
    }
    Ok(ret)
}
