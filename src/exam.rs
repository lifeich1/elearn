use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};

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
    pub clsid: i64,
}

impl ClusteringExam {
    pub fn column_count(&self) -> usize {
        let mut m: HashMap<i64, usize> = HashMap::new();
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
        let mut v: Vec<Vec<ClusteringItem>> = Vec::new();
        let mut m: BTreeMap<i64, Vec<ClusteringItem>> = BTreeMap::new();
        for p in self.data.iter() {
            if let Some(v) = m.get_mut(&p.clsid) {
                v.push(p.clone());
            } else {
                m.insert(p.clsid, vec![p.clone()]);
            }
        }
        m.values().cloned().collect()
    }
}
