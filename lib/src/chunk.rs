use crate::{
    math::{NoNeg, Point, Rect},
    utils::Float,
};
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, usize};

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

pub(crate) trait Position {
    fn position(&self) -> Point<Float>;
}

#[derive(Debug, Clone)]
enum ChunkType {
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

    pub fn values() -> [Self; 4] {
        [
            Self::FromTopLeft,
            Self::FromTopRight,
            Self::FromBottomLeft,
            Self::FromBottomRight,
        ]
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
    pub(crate) fn chunks(&self) -> Vec<(RawChunkIndex, usize)> {
        let mut result: Vec<(RawChunkIndex, usize)> = Default::default();
        for tp in ChunkType::values() {
            let rows = tp.clone().part(self);
            for y in 0..rows.len() {
                let cols = &rows[y];
                for x in 0..cols.len() {
                    let items = &rows[y][x].items;
                    result.push((
                        ChunkIndex {
                            tp: tp.clone(),
                            x,
                            y,
                        }
                        .into(),
                        items.len(),
                    ))
                }
            }
        }
        result
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

    pub(crate) fn push(&mut self, v: T)
    where
        T: Position,
    {
        let position = v.position();
        let index = RawChunkIndex::from_position::<W, H>(position).into();

        self.get_or_insert_mut(index).items.push(v);
        self.len += 1;
    }

    pub(crate) fn position<P>(&self, mut predicate: P) -> Option<Index>
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

    pub(crate) fn remove(&mut self, index: Index) -> T {
        self.len -= 1;
        self[index.chunk_index].items.remove(index.item_index)
    }

    pub(crate) fn circular_traverse_iter(
        &self,
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

    pub(crate) fn find_nearest(
        &self,
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> Option<(&T, NoNeg<Float>)>
    where
        T: Position,
    {
        self.circular_traverse_iter(position, range).find_map(
            |index| -> Option<(&T, NoNeg<Float>)> {
                if let Some(chunk) = self.get_chunk(index) {
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
                } else {
                    None
                }
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
}

#[derive(Debug, Clone)]
pub(crate) struct ChunkIndex {
    tp: ChunkType,
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
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
