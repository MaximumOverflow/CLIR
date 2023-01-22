use syn::{Data, DeriveInput, Ident};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use quote::{quote, ToTokens};
use proc_macro::TokenStream;
use syn::__private::Span;

pub fn derive(ast: DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let table_name = Ident::new(&format!("{name}Table"), Span::call_site());
	let iterator_name = Ident::new(&format!("{name}Iterator"), Span::call_site());

	let fields = match ast.data {
		Data::Struct(data) => data.fields,

		Data::Enum(_) => {
			panic!("Cannot #[derive(MetadataTable)] on an enum");
		}

		Data::Union(_) => {
			panic!("Cannot #[derive(MetadataTable)] on a union");
		}
	};

	let mut table_field_readings = HashMap::new();
	let mut table_fields = HashMap::new();

	let mut row_size = vec![];
	let mut row_parsing = vec![];
	let mut row_getters = vec![];

	for field in &fields {
		let ty = &field.ty;
		let ident = field.ident.as_ref().unwrap();

		let mut custom_reader = false;
		for attr in &field.attrs {
			let path = attr.path.to_token_stream().to_string();

			match path.as_str() {
				"table_index" => {
					custom_reader = true;
					let value = attr.tokens.to_string();
					let value = &value[1..value.len() - 1];
					let value_ident = Ident::new(value, Span::call_site());

					let field_name = value.to_case(Case::Snake);
					let field_ident = Ident::new(&field_name, Span::call_site());

					table_fields.insert(field_name.clone(), quote!(#field_ident: IndexSize));
					table_field_readings.insert(
						field_name.clone(),
						quote!(#field_ident: tables.idx_size(TableKind::#value_ident)),
					);

					row_size.push(quote!(tables.idx_size(TableKind::#value_ident) as usize));
					row_parsing.push(quote!(#ident: reader.read_table_index(self.#field_ident)?));
				}

				"coded_index" => {
					custom_reader = true;
					let value = attr.tokens.to_string();
					let value = &value[1..value.len() - 1];
					let value_ident = Ident::new(value, Span::call_site());

					let field_name = value.to_case(Case::Snake);
					let field_ident = Ident::new(&field_name, Span::call_site());

					table_fields.insert(field_name.clone(), quote!(#field_ident: IndexSize));
					table_field_readings.insert(
						field_name.clone(),
						quote!(#field_ident: CodedIndex::get_size(CodedIndexKind::#value_ident, tables)),
					);

					row_size.push(quote!(CodedIndex::get_size(CodedIndexKind::#value_ident, tables) as usize));
					row_parsing.push(quote!(#ident: reader.read_coded_index(self.#field_ident)?));
				}

				"heap_index" => {
					custom_reader = true;
					let value = attr.tokens.to_string();
					match value.as_str() {
						"(String)" => {
							table_fields.insert("str_size".to_string(), quote!(str_size: IndexSize));
							table_field_readings
								.insert("str_size".to_string(), quote!(str_size: StringHeap::idx_size(tables)));

							row_size.push(quote!(StringHeap::idx_size(tables) as usize));
							row_parsing.push(quote!(#ident: reader.read_heap_index(self.str_size)?));
						}

						"(Blob)" => {
							table_fields.insert("blob_size".to_string(), quote!(blob_size: IndexSize));
							table_field_readings
								.insert("blob_size".to_string(), quote!(blob_size: BlobHeap::idx_size(tables)));

							row_size.push(quote!(BlobHeap::idx_size(tables) as usize));
							row_parsing.push(quote!(#ident: reader.read_heap_index(self.blob_size)?));
						}

						"(Guid)" => {
							table_fields.insert("guid_size".to_string(), quote!(guid_size: IndexSize));
							table_field_readings
								.insert("guid_size".to_string(), quote!(guid_size: GuidHeap::idx_size(tables)));

							row_size.push(quote!(GuidHeap::idx_size(tables) as usize));
							row_parsing.push(quote!(#ident: reader.read_heap_index(self.guid_size)?));
						}

						_ => unimplemented!(),
					}
				}

				_ => {}
			}
		}

		if !custom_reader {
			row_size.push(quote!(std::mem::size_of::<#ty>()));
			row_parsing.push(quote!(#ident: reader.read()?));
		}

		row_getters.push(quote! {
			pub fn #ident(&self) -> #ty {
				self.#ident
			}
		});
	}

	let table_fields = table_fields.values();
	let table_field_readings = table_field_readings.values();

	let result = quote! {
		#[derive(Clone)]
		pub struct #table_name<'l> {
			bytes: &'l [u8],
			row_size: usize,
			#(#table_fields),*
		}

		#[derive(Clone)]
		pub struct #iterator_name<'l> {
			reader: ByteStream<'l>,
			table: #table_name<'l>,
		}

		impl <'l> MetadataTable<'l> for #table_name<'l> {
			type Iter = #iterator_name<'l>;

			fn bytes(&self) -> &'l [u8] {
				self.bytes
			}

			fn row_size(&self) -> usize {
				self.row_size
			}

			fn iter(&self) -> Self::Iter {
				Self::Iter {
					table: self.clone(),
					reader: ByteStream::new(self.bytes),
				}
			}
		}

		impl ParseRow for #table_name<'_> {
			type Row = #name;

			fn parse_row(&self, reader: &mut ByteStream) -> Result<Self::Row, Error> {
				Ok(Self::Row {
					#(#row_parsing),*
				})
			}
		}

		impl <'l> MetadataTableImpl<'l> for #table_name<'l> {
			fn cli_identifier() -> TableKind {
				TableKind::#name
			}

			fn calc_row_size(tables: &TableHeap) -> usize {
				#(#row_size)+*
			}

			fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
				Ok(Self {
					bytes,
					row_size: Self::calc_row_size(tables),
					#(#table_field_readings),*
				})
			}
		}

		impl Iterator for #iterator_name<'_> {
			type Item = Result<#name, Error>;

			fn next(&mut self) -> Option<Self::Item> {
				match self.reader.remaining() {
					0 => None,
					_ => Some(self.table.parse_row(&mut self.reader)),
				}
			}
		}

		impl #name {
			#(#row_getters)*
		}
	};

	result.into()
}
