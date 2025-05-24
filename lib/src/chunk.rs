use crate::{
    math::{NoNeg, Point, Rect},
    utils::Float,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, marker::PhantomData, ops::Deref, usize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Chunk<T> {
    items: Vec<T>,
}

impl<T> Default for Chunk<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<T> Chunk<T> {
    fn index_of_impl<P>(&self, predicate: &mut P) -> Option<usize>
    where
        P: FnMut(&T) -> bool,
    {
        for i in 0..self.items.len() {
            if predicate(&self.items[i]) {
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn index_of<P>(&self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(&T) -> bool,
    {
        self.index_of_impl(&mut predicate)
    }
}

pub(crate) trait Position {
    fn position(&self) -> Point<Float>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkType {
    FromTopLeft,
    FromTopRight,
    FromBottomLeft,
    FromBottomRight,
}

fn get_or_insert_mut<T, F>(v: &mut Vec<T>, i: usize, initialize: F) -> &mut T
where
    F: FnMut() -> T,
{
    if i >= v.len() {
        v.resize_with(i + 1, initialize);
    }
    &mut v[i]
}

fn remove_from_end_until<T, F>(v: &mut Vec<T>, mut pred: F)
where
    F: FnMut(&T) -> bool,
{
    let mut x = v.len() as isize - 1;
    while x >= 0 {
        if !pred(&v[x as usize]) {
            v.remove(x as usize);
            x -= 1
        } else {
            break;
        }
    }
}

impl ChunkType {
    fn part<T, const W: usize, const H: usize>(
        self,
        v: &ChunkedVec<T, W, H>,
    ) -> &Vec<Vec<Chunk<T>>> {
        match self {
            ChunkType::FromTopLeft => &v.from_top_left,
            ChunkType::FromTopRight => &v.from_top_right,
            ChunkType::FromBottomLeft => &v.from_bottom_left,
            ChunkType::FromBottomRight => &v.from_bottom_right,
        }
    }

    fn part_mut<T, const W: usize, const H: usize>(
        self,
        v: &mut ChunkedVec<T, W, H>,
    ) -> &mut Vec<Vec<Chunk<T>>> {
        match self {
            ChunkType::FromTopLeft => &mut v.from_top_left,
            ChunkType::FromTopRight => &mut v.from_top_right,
            ChunkType::FromBottomLeft => &mut v.from_bottom_left,
            ChunkType::FromBottomRight => &mut v.from_bottom_right,
        }
    }

    fn from_usize(i: usize) -> Self {
        match i {
            0 => ChunkType::FromTopLeft,
            1 => ChunkType::FromTopRight,
            2 => ChunkType::FromBottomLeft,
            3 => ChunkType::FromBottomRight,
            _ => panic!("Oops!"),
        }
    }

    #[inline]
    pub fn values() -> [Self; 4] {
        [
            Self::FromTopLeft,
            Self::FromTopRight,
            Self::FromBottomLeft,
            Self::FromBottomRight,
        ]
    }

    #[inline(always)]
    fn next(self) -> Self {
        match self {
            Self::FromTopLeft => Self::FromTopRight,
            Self::FromTopRight => Self::FromBottomLeft,
            Self::FromBottomLeft => Self::FromBottomRight,
            Self::FromBottomRight => Self::FromTopLeft,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChunkedVec<T, const W: usize, const H: usize> {
    from_top_left: Vec<Vec<Chunk<T>>>,
    from_top_right: Vec<Vec<Chunk<T>>>,
    from_bottom_left: Vec<Vec<Chunk<T>>>,
    from_bottom_right: Vec<Vec<Chunk<T>>>,
    len: usize,
}

impl<T, const W: usize, const H: usize> ChunkedVec<T, W, H> {
    pub(crate) fn chunks<'a>(&'a self) -> impl Iterator<Item = (ChunkIndex, &[T])> + 'a {
        ChunkType::values()
            .into_iter()
            .map(|tp| {
                tp.part(self)
                    .iter()
                    .enumerate()
                    .map(move |(y, rows)| {
                        rows.iter()
                            .enumerate()
                            .map(move |(x, chunk)| (ChunkIndex { tp, x, y }, chunk.items.deref()))
                    })
                    .flatten()
            })
            .flatten()
    }

    pub(crate) fn chunks_in_area<'a>(
        &'a self,
        rect: Rect<Float>,
    ) -> impl Iterator<Item = (ChunkIndex, &[T])> + 'a {
        let left_top: ChunkIndex = RawChunkIndex::from_position::<W, H>(rect.left_top()).into();
        let left_bottom: ChunkIndex =
            RawChunkIndex::from_position::<W, H>(rect.left_bottom()).into();
        let right_bottom: ChunkIndex =
            RawChunkIndex::from_position::<W, H>(rect.right_bottom()).into();
        let right_top: ChunkIndex = RawChunkIndex::from_position::<W, H>(rect.right_top()).into();

        let mut m: BTreeMap<ChunkType, Vec<ChunkIndex>> = Default::default();
        for i in [left_top, left_bottom, right_bottom, right_top] {
            m.entry(i.tp).or_insert(vec![]).push(i);
        }

        m.into_iter()
            .map(|(tp, v)| {
                let rect = if v.len() == 1 {
                    Rect::from_lrtb_unchecked(0, v[0].x, 0, v[0].y)
                } else if v.len() == 2 {
                    if v[0].x == v[1].x {
                        Rect::from_lrtb(0, v[0].x, v[0].y, v[1].y)
                    } else if v[0].y == v[1].y {
                        Rect::from_lrtb(v[0].x, v[1].x, 0, v[0].y)
                    } else {
                        panic!("Oops!")
                    }
                } else if v.len() == 4 {
                    Rect::aabb_from_points(v.into_iter().map(|v| v.point())).unwrap()
                } else {
                    panic!("Oops!")
                };

                (tp, rect)
            })
            .map(|(tp, rect)| {
                let p = tp.part(self);
                (rect.top().min(p.len())..(rect.bottom() + 1).min(p.len()))
                    .map(move |y| {
                        let p = &p[y];
                        (rect.left().min(p.len())..(rect.right() + 1).min(p.len())).map(move |x| {
                            let chunk = &p[x];
                            (ChunkIndex { tp, x, y }, chunk.items.deref())
                        })
                    })
                    .flatten()
            })
            .flatten()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|elem| f(elem));
    }

    pub(crate) fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        for tp in ChunkType::values() {
            let rows = tp.clone().part_mut(self);
            for y in 0..rows.len() {
                let cols = &mut rows[y];
                for x in 0..cols.len() {
                    let items = &mut cols[x].items;
                    for i in 0..items.len() {
                        if !f(&mut items[i]) {
                            items.remove(i);
                            self.len -= 1;
                            return;
                        }
                    }
                }
            }
        }
    }

    /// return true if any removed
    pub(crate) fn retain_by_position<F>(&mut self, position: Point<Float>, mut f: F) -> bool
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_by_position_mut(position, |elem| f(elem))
    }

    /// return true if any removed
    pub(crate) fn retain_by_position_mut<F>(&mut self, position: Point<Float>, mut f: F) -> bool
    where
        F: FnMut(&mut T) -> bool,
    {
        if let Some(chunk) =
            self.get_chunk_mut(RawChunkIndex::from_position::<W, H>(position).into())
        {
            for i in 0..chunk.items.len() {
                if !f(&mut chunk.items[i]) {
                    chunk.items.remove(i);
                    self.len -= 1;
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn push(&mut self, v: T)
    where
        T: Position,
    {
        self.get_or_insert_mut(RawChunkIndex::from_position::<W, H>(v.position()).into())
            .items
            .push(v);
        self.len += 1;
    }

    pub(crate) fn index_of<P>(&self, mut predicate: P) -> Option<Index>
    where
        P: FnMut(&T) -> bool,
    {
        for tp in ChunkType::values() {
            let rows = tp.clone().part(self);
            for y in 0..rows.len() {
                let cols = &rows[y];
                for x in 0..cols.len() {
                    let items = &rows[y][x].items;
                    for i in 0..items.len() {
                        if predicate(&items[i]) {
                            return Some(Index {
                                chunk_index: ChunkIndex {
                                    tp: tp.clone(),
                                    x,
                                    y,
                                },
                                item_index: i,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// gives index of element which satisfy predicate. but search area is limited to certain range
    pub(crate) fn index_of_in_range<P>(
        &self,
        mut predicate: P,
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> Option<Index>
    where
        P: FnMut(&T) -> bool,
    {
        Self::circular_traverse_iter(position, range).find_map(|chunk_index| {
            self.get_chunk(chunk_index.clone()).and_then(|chunk| {
                chunk.index_of_impl(&mut predicate).map(|item_index| Index {
                    chunk_index,
                    item_index,
                })
            })
        })
    }

    pub(crate) fn remove(&mut self, index: Index) -> T {
        self.len -= 1;
        self[index.chunk_index].items.remove(index.item_index)
    }

    pub(crate) fn circular_traverse_iter(
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> CircularTraverseIterator<T, W, H> {
        CircularTraverseIterator::new(position, range)
    }

    fn get_chunk(&self, i: ChunkIndex) -> Option<&Chunk<T>> {
        let part = i.tp.part(self);
        if i.y < part.len() {
            let inner_part = &part[i.y];
            if i.x < inner_part.len() {
                Some(&inner_part[i.x])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_chunk_mut(&mut self, i: ChunkIndex) -> Option<&mut Chunk<T>> {
        let part = i.tp.part_mut(self);
        if i.y < part.len() {
            let inner_part = &mut part[i.y];
            if i.x < inner_part.len() {
                Some(&mut inner_part[i.x])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn find_nearest(
        &self,
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> Option<(&T, NoNeg<Float>)>
    where
        T: Position,
    {
        Self::circular_traverse_iter(position, range).find_map(
            |chunk_index| -> Option<(&T, NoNeg<Float>)> {
                self.get_chunk(chunk_index).and_then(|chunk| {
                    chunk
                        .items
                        .iter()
                        .filter_map(|other| {
                            let dst = NoNeg::wrap((position - other.position()).len()).unwrap();
                            if dst < range {
                                Some((other, dst))
                            } else {
                                None
                            }
                        })
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                })
            },
        )
    }

    pub(crate) fn find_nearest_filter_map<'a, B, F>(
        &'a self,
        position: Point<Float>,
        range: NoNeg<Float>,
        f: F,
    ) -> Option<(B, NoNeg<Float>)>
    where
        B: Position,
        F: FnMut(&'a T) -> Option<B> + Clone,
    {
        Self::circular_traverse_iter(position, range).find_map(
            |chunk_index| -> Option<(B, NoNeg<Float>)> {
                self.get_chunk(chunk_index).and_then(|chunk| {
                    chunk
                        .items
                        .iter()
                        .filter_map(f.clone())
                        .filter_map(|other| {
                            let dst = NoNeg::wrap((position - other.position()).len()).unwrap();
                            if dst < range {
                                Some((other, dst))
                            } else {
                                None
                            }
                        })
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                })
            },
        )
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.from_top_left
            .iter()
            .chain(self.from_top_right.iter())
            .chain(self.from_bottom_left.iter())
            .chain(self.from_bottom_right.iter())
            .flatten()
            .map(|c| &c.items)
            .flatten()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.from_top_left
            .iter_mut()
            .chain(self.from_top_right.iter_mut())
            .chain(self.from_bottom_left.iter_mut())
            .chain(self.from_bottom_right.iter_mut())
            .flatten()
            .map(|c| &mut c.items)
            .flatten()
    }

    fn get_or_insert_mut(&mut self, i: ChunkIndex) -> &mut Chunk<T> {
        let part = i.tp.part_mut(self);
        let inner_part = get_or_insert_mut(part, i.y, || Default::default());
        get_or_insert_mut(inner_part, i.x, || Default::default())
    }

    /// Move all items to chunks corresponding to their position
    pub(crate) fn shuffle(&mut self)
    where
        T: Position,
    {
        let mut recipes: Vec<(T, ChunkIndex)> = Default::default();
        for tp in ChunkType::values() {
            let rows = tp.clone().part_mut(self);
            for y in 0..rows.len() {
                let cols = &mut rows[y];
                for x in 0..cols.len() {
                    let items = &mut rows[y][x].items;
                    let chunk_index = ChunkIndex {
                        tp: tp.clone(),
                        x,
                        y,
                    };

                    let mut i = 0;
                    while i < items.len() {
                        let new_chunk_index: ChunkIndex =
                            RawChunkIndex::from_position::<W, H>(items[i].position()).into();
                        if chunk_index != new_chunk_index {
                            recipes.push((items.remove(i), new_chunk_index));
                        } else {
                            i += 1
                        }
                    }
                }
            }
        }

        for (what, to_where) in recipes {
            self.get_or_insert_mut(to_where).items.push(what);
        }
    }

    pub(crate) fn collect_unused_chunks(&mut self) {
        for tp in ChunkType::values() {
            let rows = tp.clone().part_mut(self);
            for y in (0..rows.len()).rev() {
                let cols = &mut rows[y];
                remove_from_end_until(cols, |c| c.items.len() > 0);
            }
            remove_from_end_until(rows, |x| x.len() > 0);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkIndex {
    pub tp: ChunkType,
    pub x: usize,
    pub y: usize,
}

impl ChunkIndex {
    pub fn point(&self) -> Point<usize> {
        (self.x, self.y).into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawChunkIndex {
    x: isize,
    y: isize,
}

impl RawChunkIndex {
    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }

    fn from_position<const W: usize, const H: usize>(position: Point<Float>) -> Self {
        Self {
            x: (position.x().round() / W as Float).floor() as isize,
            y: (position.y().round() / H as Float).floor() as isize,
        }
    }
}

impl From<RawChunkIndex> for ChunkIndex {
    fn from(value: RawChunkIndex) -> Self {
        if value.y >= 0 {
            if value.x >= 0 {
                Self {
                    tp: ChunkType::FromTopLeft,
                    x: value.x as usize,
                    y: value.y as usize,
                }
            } else {
                Self {
                    tp: ChunkType::FromTopRight,
                    x: (-1 - value.x) as usize,
                    y: value.y as usize,
                }
            }
        } else {
            if value.x >= 0 {
                Self {
                    tp: ChunkType::FromBottomLeft,
                    x: value.x as usize,
                    y: (-1 - value.y) as usize,
                }
            } else {
                Self {
                    tp: ChunkType::FromBottomRight,
                    x: (-1 - value.x) as usize,
                    y: (-1 - value.y) as usize,
                }
            }
        }
    }
}

impl From<ChunkIndex> for RawChunkIndex {
    fn from(value: ChunkIndex) -> Self {
        match value.tp {
            ChunkType::FromTopLeft => Self {
                x: value.x as isize,
                y: value.y as isize,
            },
            ChunkType::FromTopRight => Self {
                x: -1 - (value.x as isize),
                y: value.y as isize,
            },
            ChunkType::FromBottomLeft => Self {
                x: value.x as isize,
                y: -1 - (value.y as isize),
            },
            ChunkType::FromBottomRight => Self {
                x: -1 - (value.x as isize),
                y: -1 - (value.y as isize),
            },
        }
    }
}

#[derive(Clone)]
pub(crate) struct Index {
    chunk_index: ChunkIndex,
    item_index: usize,
}

impl<T, const W: usize, const H: usize> std::ops::Index<Index> for ChunkedVec<T, W, H> {
    type Output = T;
    fn index<'a>(&'a self, i: Index) -> &'a T {
        &self[i.chunk_index].items[i.item_index]
    }
}

impl<T, const W: usize, const H: usize> std::ops::IndexMut<Index> for ChunkedVec<T, W, H> {
    fn index_mut<'a>(&'a mut self, i: Index) -> &'a mut T {
        &mut i.chunk_index.tp.part_mut(self)[i.chunk_index.y][i.chunk_index.x].items[i.item_index]
    }
}

impl<T, const W: usize, const H: usize> std::ops::Index<ChunkIndex> for ChunkedVec<T, W, H> {
    type Output = Chunk<T>;
    fn index<'a>(&'a self, i: ChunkIndex) -> &'a Chunk<T> {
        &i.tp.part(self)[i.y][i.x]
    }
}

impl<T, const W: usize, const H: usize> std::ops::IndexMut<ChunkIndex> for ChunkedVec<T, W, H> {
    fn index_mut<'a>(&'a mut self, i: ChunkIndex) -> &'a mut Chunk<T> {
        &mut i.tp.part_mut(self)[i.y][i.x]
    }
}

impl<T, const W: usize, const H: usize> FromIterator<T> for ChunkedVec<T, W, H>
where
    T: Position,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = ChunkedVec::default();
        for v in iter {
            vec.push(v);
        }
        vec
    }
}

impl<T, const W: usize, const H: usize> Default for ChunkedVec<T, W, H> {
    fn default() -> Self {
        Self {
            from_top_left: Default::default(),
            from_top_right: Default::default(),
            from_bottom_left: Default::default(),
            from_bottom_right: Default::default(),
            len: 0,
        }
    }
}

pub(crate) struct CircularTraverseIterator<T, const W: usize, const H: usize> {
    index: RawChunkIndex,
    iteration: usize,
    i: usize,
    max_i: usize,
    // rect: Rect<Float>,
    position: Point<Float>,
    range: NoNeg<Float>,
    number_of_key_skips: usize,
    _dp: PhantomData<T>,
}

impl<T, const W: usize, const H: usize> CircularTraverseIterator<T, W, H> {
    fn new(position: Point<Float>, range: NoNeg<Float>) -> Self {
        let index = RawChunkIndex::from_position::<W, H>(position);
        Self {
            index: RawChunkIndex {
                x: index.x - 1,
                y: index.y,
            },
            iteration: 0,
            i: 0,
            max_i: 0,
            position,
            range,
            number_of_key_skips: 0,
            _dp: Default::default(),
        }
    }
}

impl<T, const W: usize, const H: usize> Iterator for CircularTraverseIterator<T, W, H> {
    type Item = ChunkIndex;

    fn next(&mut self) -> Option<Self::Item> {
        enum StepResult {
            Stop,
            Skip,
            Accept(ChunkIndex),
        }

        let mut step = || -> StepResult {
            match self.iteration % 4 {
                0 => self.index.x += 1,
                1 => self.index.y += 1,
                2 => self.index.x -= 1,
                3 => self.index.y -= 1,
                _ => panic!("Undefined bahaviour"),
            };

            if self.i >= self.max_i {
                self.i = 0;
                self.max_i = self.iteration as usize / 2;
                self.iteration += 1;
            } else {
                self.i += 1;
            }

            let tile: Rect<_> = (
                self.index.x as Float * W as Float,
                self.index.y as Float * H as Float,
                W as Float,
                H as Float,
            )
                .into();
            let instersects = tile.instersects_circle(self.position, self.range);

            if self.i == 0 && !instersects {
                self.number_of_key_skips += 1;
            }

            if self.number_of_key_skips > 5 {
                StepResult::Stop
            } else {
                if instersects {
                    StepResult::Accept(self.index.clone().into())
                } else {
                    StepResult::Skip
                }
            }
        };

        loop {
            match step() {
                StepResult::Stop => break None,
                StepResult::Skip => {}
                StepResult::Accept(chunk_index) => break Some(chunk_index),
            }
        }
    }
}
