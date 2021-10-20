use rand::seq::{IteratorRandom, SliceRandom};
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// 聚类型测试数据
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClusteringExam {
    pub data: Vec<ClusteringItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClusteringItem {
    /// 表示
    pub data: String,
    /// 聚类标识
    pub clsid: i32,
}

impl ClusteringExam {
    pub fn column_count(&self) -> usize {
        let mut m: HashMap<i32, usize> = HashMap::new();
        for p in self.data.iter() {
            if let Some(v) = m.get_mut(&p.clsid) {
                *v += 1;
            } else {
                m.insert(p.clsid, 1);
            }
        }
        m.values().fold(1usize, |u, v| u.max(*v))
    }

    pub fn table(&self) -> Vec<Vec<ClusteringItem>> {
        let mut m: BTreeMap<i32, Vec<ClusteringItem>> = BTreeMap::new();
        for p in self.data.iter() {
            if let Some(v) = m.get_mut(&p.clsid) {
                v.push(p.clone());
            } else {
                m.insert(p.clsid, vec![p.clone()]);
            }
        }
        m.values().cloned().collect()
    }

    pub fn gen_probs(&self, count: usize) -> Vec<ClusteringExamProb> {
        if self.data.is_empty() {
            return Default::default();
        }
        let mut heads: Vec<ClusteringItem> = Vec::new();
        let mut rng = &mut rand::thread_rng();
        while heads.len() < count {
            let c: usize = self.data.len().min(count.wrapping_sub(heads.len()));
            heads.extend(self.data.as_slice().choose_multiple(&mut rng, c).cloned());
        }
        heads
            .iter()
            .enumerate()
            .map(|(i, h)| self.gen_prob(&mut rng, h).id_changed(i as i32))
            .collect()
    }

    fn gen_prob<R: ?Sized + rand::Rng>(
        &self,
        mut rng: &mut R,
        head: &ClusteringItem,
    ) -> ClusteringExamProb {
        let sames: Vec<ClusteringItem> = self
            .data
            .iter()
            .filter(|i| i.clsid == head.clsid)
            .cloned()
            .collect();
        let answer = sames
            .as_slice()
            .choose_multiple(&mut rng, 2)
            .filter(|i| i.data != head.data)
            .take(1)
            .cloned();
        let mut opts: Vec<ClusteringItem> = self
            .data
            .iter()
            .filter(|i| i.clsid != head.clsid)
            .choose_multiple(&mut rng, 3)
            .iter()
            .cloned()
            .cloned()
            .collect();
        opts.extend(answer);
        opts.as_mut_slice().shuffle(&mut rng);
        ClusteringExamProb {
            opts: ClusteringExamProbOption::opts_from_items(&opts[..]),
            answer: opts
                .iter()
                .enumerate()
                .find(|(_, val)| val.clsid == head.clsid)
                .expect("Should find answer in options")
                .0 as i32,
            head: head.data.clone(),
            explain: format!(
                "以下字符同类：{}",
                sames
                    .iter()
                    .fold(String::new(), |res, i| format!("{} {}", res, &i.data))
            ),
            ..Default::default()
        }
    }
}

/// 聚类型测试题目数据
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClusteringExamProb {
    pub id: i32,
    pub answer: i32,
    pub head: String,
    pub explain: String,
    pub opts: Vec<ClusteringExamProbOption>,
}

impl ClusteringExamProb {
    pub fn id_changed(self, id: i32) -> Self {
        Self { id, ..self }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ClusteringExamProbOption {
    pub id: i32,
    pub html: String,
}

impl ClusteringExamProbOption {
    pub fn opts_from_items(v: &[ClusteringItem]) -> Vec<Self> {
        v.iter()
            .enumerate()
            .map(|(i, val)| Self {
                id: i as i32,
                html: val.data.clone(),
            })
            .collect()
    }
}
