use syn::{Data, DeriveInput};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

pub fn derive(ast: DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let mut reads = vec![];
	let mut idents = vec![];
	let mut checks = vec![];

	let fields = match ast.data {
		Data::Struct(data) => data.fields,

		Data::Enum(_) => {
			panic!("Cannot #[derive(FromByteStream)] on an enum");
		}

		Data::Union(_) => {
			panic!("Cannot #[derive(FromByteStream)] on a union");
		}
	};

	for field in &fields {
		let ident = field.ident.as_ref().unwrap();
		idents.push(ident);

		let expected_value = field.attrs.iter().find_map(|attr| {
			let path = attr.path.to_token_stream().to_string();
			match path.as_str() {
				"check_value" => Some(&attr.tokens),
				"validate_value" => {
					let check = &attr.tokens;

					checks.push(quote! {
						if !(#check)(&#ident) {
							return Err(crate::raw::Error::InvalidData);
						}
					});
					None
				}
				_ => None,
			}
		});

		match expected_value {
			None => reads.push(quote!(let #ident = stream.read()?;)),
			Some(value) => reads.push(quote!(let #ident = stream.read_checked(#value)?;)),
		}
	}

	let result = quote! {
		#[allow(unused_parens)]
		impl crate::raw::FromByteStream<'_> for #name {
			fn from_byte_stream(stream: &mut crate::raw::ByteStream) -> Result<Self, crate::raw::Error> {
				#(#reads)*

				#(#checks)*

				Ok(Self { #(#idents),* })
			}
		}
	};

	result.into()
}
