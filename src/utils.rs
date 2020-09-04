pub fn even< I : std::convert::TryFrom<i8> +
                 std::ops::BitAnd<Output = I> +
                 PartialEq >
           (x : I) -> bool {
  let zero = I::try_from(0);
  let one = I::try_from(1);
  match zero {
    Ok(zero) => match one {
      Ok(one) => x & one == zero,
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
}
