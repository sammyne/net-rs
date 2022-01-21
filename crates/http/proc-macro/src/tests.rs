use quote::quote;

#[test]
fn build_handler_func() {
    let attr = quote! {};
    let input = quote! {
      async fn hello(w: &mut dyn ResponseWriter, r: Request) {}
    };

    let got = super::build_handler_func(attr, input);

    let expect = r#"fn hello < 'a > (w : & 'a mut dyn ResponseWriter , r : Request) -> :: core :: pin :: Pin < Box < :: core :: future :: Future < Output = () > + :: core :: marker :: Send + 'a > > { async fn hello (w : & mut dyn ResponseWriter , r : Request) { } Box :: pin (hello (w , r)) }"#;

    //println!("got =");
    //println!("{}", got);
    //println!();
    assert_eq!(expect.to_string(), got.to_string());
}
