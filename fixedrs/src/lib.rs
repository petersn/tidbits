pub const Q64_SHIFT: u32 = 32;

// ==================== Fixed-point 32.32 scalar ====================

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q64(pub i64);

impl Q64 {
  pub const ZERO: Self = Self(0);
  pub const EPSILON: Self = Self(1);
  pub const ONE: Self = Self(1 << Q64_SHIFT);
  pub const HALF: Self = Self(1 << (Q64_SHIFT - 1));
  pub const PI_OVER_TWO: Self = Self(6746518852);
  pub const PI: Self = Self(13493037704);
  pub const PI_TIMES_2: Self = Self(26986075409);

  pub const fn from_i32(x: i32) -> Self {
    Self((x as i64) << Q64_SHIFT)
  }

  pub fn to_i32(self) -> i32 {
    (self.0 >> Q64_SHIFT) as i32
  }

  pub fn from_f64(x: f64) -> Self {
    Self((x * (1i64 << Q64_SHIFT) as f64) as i64)
  }

  pub fn to_f64(self) -> f64 {
    self.0 as f64 / (1i64 << Q64_SHIFT) as f64
  }

  pub fn sqrt(self) -> Self {
    use num_integer::Roots;
    if self.0 < 0 {
      panic!("sqrt of negative number");
    }
    // Preserve as much precision as possible by shifting the input to the left as far as possible.
    let available_left_shift = ((self.0.leading_zeros() as u32).saturating_sub(1) / 2) * 2;
    let x = (self.0 << available_left_shift).sqrt();
    let final_shift_right = (available_left_shift as i32 - Q64_SHIFT as i32) / 2;
    if final_shift_right < 0 {
      return Self(x << -final_shift_right);
    }
    Self(x >> final_shift_right)
  }

  pub fn sin(self) -> Self {
    let mut x = Self(self.0.rem_euclid(Self::PI_TIMES_2.0));
    let mut negate = false;
    if x > Self::PI {
      x -= Self::PI;
      negate = true;
    }
    if x > Self::PI_OVER_TWO {
      x = Self::PI - x;
    }
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    const RECIP_THREE_FACT: Q64 = Q64(715827883);
    const RECIP_FIVE_FACT: Q64 = Q64(35791394);
    const RECIP_SEVEN_FACT: Q64 = Q64(852176);
    let mut y = x - x3 * RECIP_THREE_FACT + x5 * RECIP_FIVE_FACT - x7 * RECIP_SEVEN_FACT;
    if negate {
      y = -y;
    }
    y
  }

  pub fn cos(self) -> Self {
    (Self::PI_OVER_TWO - self).sin()
  }

  pub fn angle_to_vec(self) -> Q64Vec {
    // FIXME: I can compute these two simultaneously more efficiently.
    Q64Vec {
      x: self.cos(),
      y: self.sin(),
    }
  }

  pub fn abs(self) -> Self {
    Self(self.0.abs())
  }

  pub fn ceiling_div_by_positive(self, rhs: Self) -> Self {
    if rhs.0 <= 0 {
      panic!("rhs must be positive");
    }
    let lhs = (self.0 as i128) << Q64_SHIFT;
    Self(((lhs + rhs.0 as i128 - 1) / rhs.0 as i128) as i64)
  }
}

impl std::ops::Neg for Q64 {
  type Output = Self;
  fn neg(self) -> Self {
    Self(-self.0)
  }
}

impl std::ops::Add for Q64 {
  type Output = Self;
  fn add(self, rhs: Self) -> Self {
    Self(self.0 + rhs.0)
  }
}

impl std::ops::Sub for Q64 {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self {
    Self(self.0 - rhs.0)
  }
}

impl std::ops::Mul for Q64 {
  type Output = Self;
  fn mul(self, rhs: Self) -> Self {
    Self(((self.0 as i128 * rhs.0 as i128) >> Q64_SHIFT) as i64)
  }
}

impl std::ops::Div for Q64 {
  type Output = Self;
  fn div(self, rhs: Self) -> Self {
    Self((((self.0 as i128) << Q64_SHIFT) / rhs.0 as i128) as i64)
  }
}

impl std::ops::Shl<u32> for Q64 {
  type Output = Self;
  fn shl(self, shift: u32) -> Self {
    Self(self.0 << shift)
  }
}

impl std::ops::Shr<u32> for Q64 {
  type Output = Self;
  fn shr(self, shift: u32) -> Self {
    Self(self.0 >> shift)
  }
}

impl std::ops::AddAssign for Q64 {
  fn add_assign(&mut self, rhs: Self) {
    *self = *self + rhs;
  }
}

impl std::ops::SubAssign for Q64 {
  fn sub_assign(&mut self, rhs: Self) {
    *self = *self - rhs;
  }
}

impl std::ops::MulAssign for Q64 {
  fn mul_assign(&mut self, rhs: Self) {
    *self = *self * rhs;
  }
}

impl std::ops::DivAssign for Q64 {
  fn div_assign(&mut self, rhs: Self) {
    *self = *self / rhs;
  }
}

impl std::ops::ShlAssign<u32> for Q64 {
  fn shl_assign(&mut self, shift: u32) {
    *self = *self << shift;
  }
}

impl std::ops::ShrAssign<u32> for Q64 {
  fn shr_assign(&mut self, shift: u32) {
    *self = *self >> shift;
  }
}

// We serialize and deserialize numbers as strings to avoid precision issues with JSON.

impl serde::Serialize for Q64 {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0.to_string())
  }
}

impl<'de> serde::Deserialize<'de> for Q64 {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    Ok(Self(s.parse::<i64>().map_err(serde::de::Error::custom)?))
  }
}

// ==================== Fixed-point vector ====================

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q64Vec {
  pub x: Q64,
  pub y: Q64,
}

impl Q64Vec {
  pub const ZERO: Self = Self {
    x: Q64::ZERO,
    y: Q64::ZERO,
  };

  pub fn new(x: Q64, y: Q64) -> Self {
    Self { x, y }
  }

  // Based on https://stackoverflow.com/a/14100975/3047059
  pub fn get_angle(self) -> Q64 {
    const B: Q64 = Q64(2560775466); // 0.596227
    let x_sign = self.x < Q64::ZERO;
    let y_sign = self.y < Q64::ZERO;
    let q = 4 * (!x_sign & y_sign) as i32 + 2 * x_sign as i32;
    let bxy_a = (B * self.x * self.y).abs();
    let num = bxy_a + self.y * self.y;
    let denom = self.x * self.x + bxy_a + num;
    if denom == Q64::ZERO {
      return Q64::ZERO;
    }
    let mut atan_1q = num / denom;
    if x_sign != y_sign && atan_1q > Q64::ZERO {
      atan_1q = -atan_1q;
    }
    (Q64::from_i32(q) + atan_1q) * Q64::PI_OVER_TWO
  }

  pub fn from_i32(x: i32, y: i32) -> Self {
    Self {
      x: Q64::from_i32(x),
      y: Q64::from_i32(y),
    }
  }

  pub fn from_f64(x: f64, y: f64) -> Self {
    Self {
      x: Q64::from_f64(x),
      y: Q64::from_f64(y),
    }
  }

  pub fn to_f64(self) -> (f64, f64) {
    (self.x.to_f64(), self.y.to_f64())
  }

  pub fn norm_squared(self) -> Q64 {
    self.x * self.x + self.y * self.y
  }

  pub fn norm(self) -> Q64 {
    // Avoid overflow issues by simply returning a large value if any coordinate is large enough.
    if self.x.abs() > Q64::from_i32(30_000) || self.y.abs() > Q64::from_i32(30_000) {
      return Q64::from_i32(1_000_000);
    }
    (self.x * self.x + self.y * self.y).sqrt()
  }

  pub fn normalized(self) -> Self {
    let norm = self.norm();
    match norm {
      Q64::ZERO => Self::ZERO,
      _ => self / norm,
    }
  }
}

impl std::ops::Neg for Q64Vec {
  type Output = Self;
  fn neg(self) -> Self {
    Self {
      x: -self.x,
      y: -self.y,
    }
  }
}

impl std::ops::Add for Q64Vec {
  type Output = Self;
  fn add(self, rhs: Self) -> Self {
    Self {
      x: self.x + rhs.x,
      y: self.y + rhs.y,
    }
  }
}

impl std::ops::Sub for Q64Vec {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self {
    Self {
      x: self.x - rhs.x,
      y: self.y - rhs.y,
    }
  }
}

impl std::ops::Mul<Q64> for Q64Vec {
  type Output = Self;
  fn mul(self, rhs: Q64) -> Self {
    Self {
      x: self.x * rhs,
      y: self.y * rhs,
    }
  }
}

impl std::ops::Mul<Q64Vec> for Q64 {
  type Output = Q64Vec;
  fn mul(self, rhs: Q64Vec) -> Q64Vec {
    rhs * self
  }
}

impl std::ops::Div<Q64> for Q64Vec {
  type Output = Self;
  fn div(self, rhs: Q64) -> Self {
    Self {
      x: self.x / rhs,
      y: self.y / rhs,
    }
  }
}

impl std::ops::AddAssign for Q64Vec {
  fn add_assign(&mut self, rhs: Self) {
    *self = *self + rhs;
  }
}

impl std::ops::SubAssign for Q64Vec {
  fn sub_assign(&mut self, rhs: Self) {
    *self = *self - rhs;
  }
}

impl std::ops::MulAssign<Q64> for Q64Vec {
  fn mul_assign(&mut self, rhs: Q64) {
    *self = *self * rhs;
  }
}

impl std::ops::DivAssign<Q64> for Q64Vec {
  fn div_assign(&mut self, rhs: Q64) {
    *self = *self / rhs;
  }
}

impl serde::Serialize for Q64Vec {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{},{}", self.x.0, self.y.0))
  }
}

impl<'de> serde::Deserialize<'de> for Q64Vec {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    let s = String::deserialize(deserializer)?;
    let mut parts = s.split(',');
    let mut get_val = || Ok(
      parts.next()
        .ok_or_else(|| serde::de::Error::custom("missing part"))?
        .parse::<i64>()
        .map_err(serde::de::Error::custom)?
    );
    let x = get_val()?;
    let y = get_val()?;
    if parts.next().is_some() {
      return Err(serde::de::Error::custom("too many parts"));
    }
    Ok(Self::new(Q64(x), Q64(y)))
  }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_square_roots() {
    for i in 0..1000 {
      let x = Q64::from_i32(i);
      let actual_square_root = x.to_f64().sqrt();
      let computed_square_root = x.sqrt().to_f64();
      assert!((actual_square_root - computed_square_root).abs() < 1e-6);
    }
  }

  #[test]
  fn test_sin_cos() {
    for i in 0..1000 {
      let x = Q64::from_i32(i);
      let actual_sin = x.to_f64().sin();
      let computed_sin = x.sin().to_f64();
      assert!((actual_sin - computed_sin).abs() < 1e-3);
      let actual_cos = x.to_f64().cos();
      let computed_cos = x.cos().to_f64();
      assert!((actual_cos - computed_cos).abs() < 1e-3);
    }
  }

  #[test]
  fn test_angle_to_vec() {
    for x in -50..50 {
      for y in -50..50 {
        let v = Q64Vec::from_i32(x, y);
        let mut actual_atan2 = v.y.to_f64().atan2(v.x.to_f64());
        if actual_atan2 < 0.0 {
          actual_atan2 += 2.0 * std::f64::consts::PI;
        }
        let computed_atan2 = v.get_angle().to_f64();
        assert!((actual_atan2 - computed_atan2).abs() < 5e-3);
      }
    }
  }

  #[test]
  fn test_serialization() {
    let v = Q64Vec::from_f64(123.456, -789.012);
    let s = serde_json::to_string(&v).unwrap();
    let v2: Q64Vec = serde_json::from_str(&s).unwrap();
    assert_eq!(v, v2);
  }
}
