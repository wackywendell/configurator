#[macro_export]
macro_rules! configurator {
    ($config_name:ident {
        $($attr_fixed:ident : $attr_fixed_type:ty, $fixed_doc:expr),*;
        $($attr_def:ident : $attr_def_type:ty = $def:expr, $def_doc:expr),*
    }) => {
        #[derive(Debug)]
        pub struct $config_name {
            $(#[doc = $fixed_doc] pub $attr_fixed : $attr_fixed_type),*,
            $(#[doc = $def_doc] pub $attr_def : Option<$attr_def_type>),*
        }
    }
}

configurator!(
    MyConfig {
        x : isize, "x option";
        y : isize = 4, "y option"
    }
);

#[test]
fn it_works() {
    let c = MyConfig {
        x:3,
        y: Some(4)
    };
    println!("{:?}", c);
    
    assert!(false);
}
