//! UI-neutral, fallible ordering shared by table delegates and query handlers.

use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap, fmt, sync::Arc};

/// Maximum number of clauses accepted by a sort order.
pub const MAX_SORT_CLAUSES: usize = 32;

/// Direction of a non-null comparison.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Null placement is independent of ascending or descending direction.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NullPlacement {
    First,
    #[default]
    Last,
}

/// One ordering key, identified by its stable table column key.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SortClause {
    column: String,
    direction: SortDirection,
    #[serde(default)]
    nulls: NullPlacement,
}

impl SortClause {
    pub fn new(column: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
            nulls: NullPlacement::Last,
        }
    }

    pub fn with_nulls(mut self, nulls: NullPlacement) -> Self {
        self.nulls = nulls;
        self
    }

    pub fn column(&self) -> &str {
        &self.column
    }
    pub const fn direction(&self) -> SortDirection {
        self.direction
    }
    pub const fn nulls(&self) -> NullPlacement {
        self.nulls
    }
}

/// Ordered clauses. Equal keys retain source order unless a row identity breaks the tie.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SortOrder(Vec<SortClause>);

impl<'de> Deserialize<'de> for SortOrder {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(Vec::<SortClause>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl SortOrder {
    pub fn new(clauses: Vec<SortClause>) -> Result<Self, SortError> {
        if clauses.len() > MAX_SORT_CLAUSES {
            return Err(SortError::TooManyClauses);
        }
        for (ix, clause) in clauses.iter().enumerate() {
            if clause.column.is_empty() || clause.column.trim() != clause.column {
                return Err(SortError::InvalidColumn(clause.column.clone()));
            }
            if clauses[..ix]
                .iter()
                .any(|previous| previous.column == clause.column)
            {
                return Err(SortError::DuplicateColumn(clause.column.clone()));
            }
        }
        Ok(Self(clauses))
    }

    pub fn clauses(&self) -> &[SortClause] {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Validate even when the source has no rows.
    pub fn validate<R: SortableRow>(&self) -> Result<(), SortError> {
        for clause in &self.0 {
            if !R::sortable_columns().contains(&clause.column()) {
                return Err(SortError::UnsupportedColumn(clause.column.clone()));
            }
        }
        Ok(())
    }

    /// Build a permutation without mutating rows. Comparison failures are atomic.
    pub fn sorted_indices<R: SortableRow>(&self, rows: &[R]) -> Result<Vec<usize>, SortError> {
        let mut indices = (0..rows.len()).collect::<Vec<_>>();
        self.sort_indices(rows, &mut indices)?;
        Ok(indices)
    }

    /// Sort selected source indices atomically, without comparing excluded rows.
    pub fn sort_indices<R: SortableRow>(
        &self,
        rows: &[R],
        indices: &mut [usize],
    ) -> Result<(), SortError> {
        self.validate::<R>()?;
        sort_indices_by(self, rows, indices, &|left, right, clause| {
            left.compare_sort_clause(right, clause)
        })
    }

    /// Order a complete source before pagination. Errors leave it unchanged.
    pub fn sort_rows<R: SortableRow>(&self, rows: &mut Vec<R>) -> Result<(), SortError> {
        let indices = self.sorted_indices(rows)?;
        apply_permutation(rows, indices);
        Ok(())
    }
}

/// Typed row comparisons. Generated rows implement this without GPUI state.
pub trait SortableRow {
    fn sortable_columns() -> &'static [&'static str];
    fn compare_sort_clause(&self, other: &Self, clause: &SortClause)
    -> Result<Ordering, SortError>;

    /// An optional stable, ascending identity tie-breaker supplied by the row owner.
    fn compare_sort_identity(&self, _other: &Self) -> Result<Ordering, SortError> {
        Ok(Ordering::Equal)
    }
}

type ResolvedComparison<R> =
    dyn Fn(&R, &R, &SortClause) -> Result<Ordering, SortError> + Send + Sync;

/// One application-resolved stable key with an executable typed calculation.
/// The owning application resolves vocabulary, authorization and source readiness.
pub struct ResolvedSortColumn<R> {
    key: String,
    compare: Arc<ResolvedComparison<R>>,
}
impl<R> Clone for ResolvedSortColumn<R> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            compare: self.compare.clone(),
        }
    }
}
impl<R> fmt::Debug for ResolvedSortColumn<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedSortColumn")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}
impl<R> ResolvedSortColumn<R> {
    /// Use the same typed extraction in ordering and the application's cell presentation.
    /// Each selected row is self-compared before sorting, including a single-row source.
    pub fn new<K: PartialOrd, F>(key: impl Into<String>, extract: F) -> Result<Self, SortError>
    where
        F: Fn(&R) -> Result<Option<K>, SortError> + Send + Sync + 'static,
    {
        let key = key.into();
        SortOrder::new(vec![SortClause::new(key.clone(), SortDirection::Ascending)])?;
        Ok(Self {
            key,
            compare: Arc::new(move |left, right, clause| {
                let left = extract(left)?;
                let right = extract(right)?;
                compare_values(left.as_ref(), right.as_ref(), clause)
            }),
        })
    }
    /// Stable application-resolved column key.
    pub fn key(&self) -> &str {
        &self.key
    }
}

/// Resolved additions to the row's declared sortable keys, retained by the source owner.
/// Native keys and resolved keys use one fallible, atomic ordering implementation.
pub struct ResolvedSortColumns<R> {
    columns: BTreeMap<String, ResolvedSortColumn<R>>,
}
impl<R> Default for ResolvedSortColumns<R> {
    fn default() -> Self {
        Self {
            columns: BTreeMap::new(),
        }
    }
}
impl<R> Clone for ResolvedSortColumns<R> {
    fn clone(&self) -> Self {
        Self {
            columns: self.columns.clone(),
        }
    }
}
impl<R> fmt::Debug for ResolvedSortColumns<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedSortColumns")
            .field("keys", &self.columns.keys().collect::<Vec<_>>())
            .finish()
    }
}
impl<R: SortableRow> ResolvedSortColumns<R> {
    /// Bind resolved keys, rejecting duplicates and collisions with native row keys.
    pub fn new(
        columns: impl IntoIterator<Item = ResolvedSortColumn<R>>,
    ) -> Result<Self, SortError> {
        let mut resolved = Self::default();
        for column in columns {
            if R::sortable_columns().contains(&column.key())
                || resolved.columns.contains_key(column.key())
            {
                return Err(SortError::DuplicateColumn(column.key().into()));
            }
            resolved.columns.insert(column.key.clone(), column);
        }
        Ok(resolved)
    }
    /// Registered resolved keys in stable lexical order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.columns.keys().map(String::as_str)
    }
    /// Validate capabilities even for an empty source.
    pub fn validate(&self, order: &SortOrder) -> Result<(), SortError> {
        for clause in order.clauses() {
            if !R::sortable_columns().contains(&clause.column())
                && !self.columns.contains_key(clause.column())
            {
                return Err(SortError::UnsupportedColumn(clause.column().into()));
            }
        }
        Ok(())
    }
    /// Order selected source indices atomically, preflighting only included rows.
    pub fn sort_indices(
        &self,
        order: &SortOrder,
        rows: &[R],
        indices: &mut [usize],
    ) -> Result<(), SortError> {
        self.validate(order)?;
        sort_indices_by(order, rows, indices, &|left, right, clause| match self
            .columns
            .get(clause.column())
        {
            Some(column) => (column.compare)(left, right, clause),
            None => left.compare_sort_clause(right, clause),
        })
    }
    /// Build the complete ordered source permutation before pagination.
    pub fn sorted_indices(&self, order: &SortOrder, rows: &[R]) -> Result<Vec<usize>, SortError> {
        let mut indices = (0..rows.len()).collect::<Vec<_>>();
        self.sort_indices(order, rows, &mut indices)?;
        Ok(indices)
    }
    /// Order the complete source; a failed calculation leaves rows unchanged.
    pub fn sort_rows(&self, order: &SortOrder, rows: &mut Vec<R>) -> Result<(), SortError> {
        let indices = self.sorted_indices(order, rows)?;
        apply_permutation(rows, indices);
        Ok(())
    }
}

fn apply_permutation<R>(rows: &mut Vec<R>, indices: Vec<usize>) {
    let mut source = std::mem::take(rows)
        .into_iter()
        .map(Some)
        .collect::<Vec<_>>();
    rows.extend(
        indices
            .into_iter()
            .map(|ix| source[ix].take().expect("sort permutation")),
    );
}
fn sort_indices_by<R: SortableRow>(
    order: &SortOrder,
    rows: &[R],
    indices: &mut [usize],
    compare: &impl Fn(&R, &R, &SortClause) -> Result<Ordering, SortError>,
) -> Result<(), SortError> {
    if let Some(ix) = indices.iter().find(|ix| **ix >= rows.len()) {
        return Err(SortError::InvalidRowIndex(*ix));
    }
    if order.is_empty() {
        return Ok(());
    }
    for ix in indices.iter().copied() {
        for clause in order.clauses() {
            if compare(&rows[ix], &rows[ix], clause)? != Ordering::Equal {
                return Err(SortError::UnorderedColumn(clause.column().into()));
            }
        }
    }
    let mut ordered = indices.to_vec();
    let mut scratch = ordered.clone();
    merge_sort(&mut ordered, &mut scratch, &|left, right| {
        for clause in order.clauses() {
            let compared = compare(&rows[left], &rows[right], clause)?;
            if compared != Ordering::Equal {
                return Ok(compared);
            }
        }
        rows[left].compare_sort_identity(&rows[right])
    })?;
    indices.copy_from_slice(&ordered);
    Ok(())
}

/// Compare typed values, including optional calculated keys.
/// Unordered values such as NaN are errors; they are never silently treated as ties.
pub fn compare_values<T: PartialOrd + ?Sized>(
    left: Option<&T>,
    right: Option<&T>,
    clause: &SortClause,
) -> Result<Ordering, SortError> {
    Ok(match (left, right) {
        (Some(left), Some(right)) => {
            let order = left
                .partial_cmp(right)
                .ok_or_else(|| SortError::UnorderedColumn(clause.column.clone()))?;
            match clause.direction {
                SortDirection::Ascending => order,
                SortDirection::Descending => order.reverse(),
            }
        },
        (None, None) => Ordering::Equal,
        (None, Some(_)) => match clause.nulls {
            NullPlacement::First => Ordering::Less,
            NullPlacement::Last => Ordering::Greater,
        },
        (Some(_), None) => match clause.nulls {
            NullPlacement::First => Ordering::Greater,
            NullPlacement::Last => Ordering::Less,
        },
    })
}

/// Invalid ordering input or a failed typed comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortError {
    TooManyClauses,
    InvalidRowIndex(usize),
    InvalidColumn(String),
    DuplicateColumn(String),
    UnsupportedColumn(String),
    UnorderedColumn(String),
    Calculation { column: String, message: String },
}

impl fmt::Display for SortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRowIndex(ix) => write!(f, "invalid source row index {ix}"),
            Self::TooManyClauses => write!(f, "sort order exceeds {MAX_SORT_CLAUSES} clauses"),
            Self::InvalidColumn(column) => write!(f, "invalid sort column `{column}`"),
            Self::DuplicateColumn(column) => write!(f, "duplicate sort column `{column}`"),
            Self::UnsupportedColumn(column) => write!(f, "unsupported sort column `{column}`"),
            Self::Calculation { column, message } => {
                write!(f, "calculated sort key `{column}` failed: {message}")
            },
            Self::UnorderedColumn(column) => {
                write!(f, "unordered values in sort column `{column}`")
            },
        }
    }
}
impl std::error::Error for SortError {}

fn merge_sort(
    indices: &mut [usize],
    scratch: &mut [usize],
    compare: &impl Fn(usize, usize) -> Result<Ordering, SortError>,
) -> Result<(), SortError> {
    if indices.len() < 2 {
        return Ok(());
    }
    let mid = indices.len() / 2;
    let (left, right) = indices.split_at_mut(mid);
    let (left_scratch, right_scratch) = scratch.split_at_mut(mid);
    merge_sort(left, left_scratch, compare)?;
    merge_sort(right, right_scratch, compare)?;
    let (mut left_ix, mut right_ix) = (0, mid);
    for slot in scratch.iter_mut() {
        if left_ix < mid
            && (right_ix == indices.len()
                || compare(indices[left_ix], indices[right_ix])? != Ordering::Greater)
        {
            *slot = indices[left_ix];
            left_ix += 1;
        } else {
            *slot = indices[right_ix];
            right_ix += 1;
        }
    }
    indices.copy_from_slice(scratch);
    Ok(())
}

/// A calculated key can distinguish a null result from an evaluation failure.
pub trait IntoSortKey {
    type Key: PartialOrd;
    fn into_sort_key(self) -> Result<Option<Self::Key>, SortError>;
}
impl<Key: PartialOrd> IntoSortKey for Option<Key> {
    type Key = Key;
    fn into_sort_key(self) -> Result<Option<Key>, SortError> {
        Ok(self)
    }
}
impl<Key: PartialOrd> IntoSortKey for Result<Option<Key>, SortError> {
    type Key = Key;
    fn into_sort_key(self) -> Result<Option<Key>, SortError> {
        self
    }
}
