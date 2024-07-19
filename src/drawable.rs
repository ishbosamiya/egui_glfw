// TODO: need to eventually get rid of [`Drawable`] completely, it is
// a terrible way to implement drawing/rendering. Having custom
// methods that get called in a chain is just strictly better. This
// trait achieves something similar but with extra steps that
// introduces complexity.
pub trait Drawable<ExtraData, Error> {
    fn draw(&self, extra_data: &mut ExtraData) -> Result<(), Error>;
}
