macro_rules! db_add {
  ($name:ident,$new:ident, $result:ident, $table:ident ) => {
    pub fn $name(&mut self, new: $new) {
      let result: $result = diesel::insert_into($table::table)
        .values(&new)
        .get_result(&self.get_connection())
        .expect("Error saving new $ident");
      self.$table.push(result);
    }
  };
}

macro_rules! hashmap {
  ($( $key: expr => $val: expr ),*) => {{
       let mut map = ::std::collections::HashMap::new();
       $( map.insert($key, $val); )*
       map
  }}
}
