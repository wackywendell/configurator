#[macro_export]
macro_rules! configurator {
    ($config_name:ident {
        $($attr_fixed:ident : $attr_fixed_type:ty),*;
        $($attr_def:ident : $attr_def_type:ty = $def:expr),*
    }) => {
        #[derive(Debug)]
        struct $config_name {
            $($attr_fixed : $attr_fixed_type),*,
            $($attr_def : Option<$attr_def_type>),*
        }
    }
}


macro_rules! config {
    ($config_name:ident {
        $attr_fixed:ident : $attr_fixed_type:ty
    }) => {
        #[derive(Debug)]
        struct $config_name {
            $attr_fixed : $attr_fixed_type,
        }
    }
}

configurator!(
    MyConfig {
        x : isize;
        y : isize = 4
    }
);

#[test]
fn it_works() {
    let c = MyConfig {
        x:3,
        y: Some(4)
    };
    println!("{:?}", c);
}
