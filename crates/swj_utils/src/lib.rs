extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Token, Type};
use syn::parse::{Parse, ParseStream};


struct UnitTeamSystem {
    teams: Vec<Type>,
    systems: Vec<Ident>,
}

impl Parse for UnitTeamSystem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut teams = Vec::new();
        let mut systems = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![;]) {
                break;
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            let team: Type = input.parse()?;
            teams.push(team);
        }

        input.parse::<Token![;]>()?;

        while !input.is_empty() {
            let system: Ident = input.parse()?;
            systems.push(system);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(UnitTeamSystem {
            teams,
            systems,
        })
    }
}

#[proc_macro]
pub fn unit_team_system(input: TokenStream) -> TokenStream {
    let UnitTeamSystem { teams, systems } = syn::parse_macro_input!(input as UnitTeamSystem);

    let mut tokens = Vec::with_capacity(teams.len());

    for team in teams {
        for system in &systems {
            tokens.push(quote! {
                #system::<#team>,
            });
        }
    }

    TokenStream::from(quote! {
        (#(#tokens)*)
    })
}