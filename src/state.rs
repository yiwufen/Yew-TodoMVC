
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use yew::{html::IntoPropValue, Html};
use yew::html;

pub struct State {
    pub entries: Vec<Entry>,
    pub filter: Filter,
    pub edit_value: String,
}

impl State {
    pub fn total(&self) -> usize {
        self.entries.len()
    }
    /// 是否所有Entry全部完成
    pub fn is_all_completed(&self) -> bool {
        let mut filtered_iter = self.entries.iter()
        .filter(|e| self.filter.fits(e))
        .peekable();
        if filtered_iter.peek().is_none() {
            return false;
        }
        filtered_iter.all(|e| e.completed)
    }

    pub fn toggle_all(&mut self, value: bool) {
        for entry in &mut self.entries {
            if self.filter.fits(entry) {
                entry.completed = value;
            }
        }
    }
    pub fn toggle(&mut self, idx: usize) {
        let filter = self.filter;
        let entry = self
            .entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .nth(idx)
            .unwrap();
        entry.completed = !entry.completed;
    }
    pub fn toggle_edit(&mut self, idx: usize) {
        let filter = self.filter;
        let entry = self
            .entries
            .iter_mut()
            .filter(|e| filter.fits(e))
            .nth(idx)
            .unwrap();
        entry.editing = !entry.editing;
    }
    pub fn remove(&mut self, idx: usize) {
        let idx = {
            let entries = &self.entries
                .iter()
                .enumerate()
                .filter(|&(_, e)| self.filter.fits(e))
                .collect::<Vec<_>>();
            let &(idx, _) = entries.get(idx).unwrap();
            idx
        };  
        self.entries.remove(idx);
    }

    pub fn complete_edit(&mut self, idx: usize, val: String) {
        if val.is_empty() {
            self.remove(idx);
        } else {
            let filter = self.filter;
            let entry = self
                .entries
                .iter_mut()
                .filter(|e| filter.fits(e))
                .nth(idx)
                .unwrap();
            entry.description = val;
            entry.editing = !entry.editing;
        }

    }
    pub fn clear_completed(&mut self) {
        self.entries.retain(|e| !e.completed);
    }
    pub fn completed_count(&self) -> usize {
        self.entries.iter().filter(|e| Filter::Completed.fits(e)).count()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Entry {
    pub description: String,
    pub completed: bool,
    pub editing: bool,
}

#[derive(Clone, Copy, Debug, EnumIter, Display, PartialEq, Serialize, Deserialize, Eq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    pub fn fits(&self, entry: &Entry) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !entry.completed,
            Filter::Completed => entry.completed,
        }
    }
    pub fn as_href(&self) -> &'static str {
        match self {
            Filter::All => "#/",
            Filter::Active => "#/active",
            Filter::Completed => "#/completed",
        }
    }
}

impl IntoPropValue<Html> for Filter {
    fn into_prop_value(self) -> Html {
        html! {
        <>
        {self.to_string()}
        </>
        }
    }
}