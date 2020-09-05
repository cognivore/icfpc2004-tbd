use num_traits::FromPrimitive;


// TODO: Abstract away
// ...
// TODO

pub fn simple_enum_iter<T: FromPrimitive>(n : i8) -> impl Iterator<Item=T> {
    (0..n).map(|x| FromPrimitive::from_i8(x).unwrap())
}

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

// ENTRY_POINT
pub fn prelude_entry_point() {
    println!("Hello from prelude")
}
