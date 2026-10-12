///|
fn assert_native_build_token(token : String) -> Unit raise {
  assert_eq(token.length(), 32)
  for character in token.iter() {
    assert_true(
      ('0' <= character && character <= '9') ||
      ('a' <= character && character <= 'f'),
    )
  }
}

///|
test "native build token preserves every byte including zero-padded nibbles" {
  let bytes = b"\x00\x01\x0f\x10\xff\x00\x01\x0f\x10\xff\x00\x01\x0f\x10\xff\x00"
  let token = native_build_token(bytes)
  assert_eq(token, "00010f10ff00010f10ff00010f10ff00")
  assert_native_build_token(token)
}

///|
test "actual platform entropy yields fresh 128-bit native build tokens" {
  let first = match @env.rand(16) {
    Some(bytes) => bytes
    None => fail("Native build producer requires platform entropy")
  }
  let second = match @env.rand(16) {
    Some(bytes) => bytes
    None => fail("Native build producer requires platform entropy")
  }
  assert_eq(first.length(), 16)
  assert_eq(second.length(), 16)
  let first_token = native_build_token(first)
  let second_token = native_build_token(second)
  assert_native_build_token(first_token)
  assert_native_build_token(second_token)
  assert_true(first_token != second_token)
}

///|
test "production build environment preserves inheritance and overrides old tokens" {
  let token_key = "VIZE_NATIVE_BUILD_TOKEN"
  let probe_key = "VIZE_NATIVE_BUILD_TOKEN_WBTEST_PROBE"
  let empty_key = "VIZE_NATIVE_BUILD_TOKEN_WBTEST_EMPTY"
  let original_token = @env.get_env_var(token_key)
  let original_probe = @env.get_env_var(probe_key)
  let original_empty = @env.get_env_var(empty_key)
  let inherited_token = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  @env.set_env_var(token_key, inherited_token)
  @env.set_env_var(probe_key, "native build inheritance 日本語\nsecond line")
  @env.set_env_var(empty_key, "")
  let inherited = @env.get_env_vars()
  let first = match native_build_environment() {
    Some(environment) => environment
    None => fail("Production native build environment requires entropy")
  }
  let second = match native_build_environment() {
    Some(environment) => environment
    None => fail("Production native build environment requires entropy")
  }
  assert_eq(first.length(), inherited.length())
  assert_eq(second.length(), inherited.length())
  for key, value in inherited {
    if key != token_key {
      // Do not print inherited environment values in test failure output.
      assert_true(first.get(key) == Some(value))
      assert_true(second.get(key) == Some(value))
    }
  }
  let first_token = first.get(token_key).unwrap()
  let second_token = second.get(token_key).unwrap()
  assert_native_build_token(first_token)
  assert_native_build_token(second_token)
  assert_true(first_token != inherited_token)
  assert_true(second_token != inherited_token)
  assert_true(first_token != second_token)
  assert_eq(@env.get_env_var(token_key), Some(inherited_token))
  let originals = [
    (token_key, original_token),
    (probe_key, original_probe),
    (empty_key, original_empty),
  ]
  for entry in originals {
    let (key, original) = entry
    match original {
      Some(value) => @env.set_env_var(key, value)
      None => @env.unset_env_var(key)
    }
  }
}
