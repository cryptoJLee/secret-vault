use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, Querier, StdError,
    StdResult, Storage,
};
use secret_toolkit::crypto::secp256k1::PrivateKey;

use crate::msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
use crate::responses::{CreateKeyResponse, SignResponse};
use crate::state::{get_key_record, get_seed, store_key_record, store_seed};
use crate::utils::{
    authenticate_request, generate_api_key, generate_key_id, generate_private_key, generate_seed,
    validate_data_len,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let seed = match msg {
        InitMsg::Init { seed_phrase } => generate_seed(&seed_phrase),
    };

    store_seed(&mut (deps.storage), seed);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let result: HandleResponse = match msg {
        HandleMsg::NewKey {
            key_seed,
            passphrase,
        } => {
            let seed = get_seed(&mut (deps.storage));

            let key_id = generate_key_id(&env);

            let api_key = generate_api_key(&seed, &env);

            let private_key = generate_private_key(&env, &seed, &key_seed.into_bytes());

            store_key_record(
                &mut (deps.storage),
                &key_id,
                private_key,
                api_key.clone(),
                passphrase,
            );

            let public_key = PrivateKey::parse(&private_key)?
                .pubkey()
                .serialize_compressed();

            CreateKeyResponse {
                api_key,
                key_id,
                public_key,
            }
            .into()
        }
        HandleMsg::Sign {
            api_key,
            key_id,
            data,
            passphrase,
        } => {
            // let record_bytes = deps.storage.get(&key_id.as_bytes()).unwrap();

            let record = get_key_record(&mut (deps.storage), &key_id)?;

            if !authenticate_request(&record, &api_key, &passphrase) {
                return Err(StdError::generic_err(
                    "Unauthorized. Bad API key or passphrase",
                ));
            }

            let data_bytes = hex::decode(data).map_err(|_| {
                StdError::generic_err("Error validating data format: should be hex string")
            })?;

            if !validate_data_len(&data_bytes) {
                return Err(StdError::generic_err(
                    "Error validating data size: Should be 64 characters",
                ));
            }

            let mut data_arr = [0u8; 32];
            data_arr.copy_from_slice(&data_bytes);

            let signature = PrivateKey::parse(&record.key)?.sign(&data_arr).serialize();

            SignResponse { signature }.into()
        }
    };

    Ok(result)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    Err(StdError::generic_err("Queries are not supported yet :)"))
}

/////////////////////////////// Migrate ///////////////////////////////
// Isn't supported by the Secret Network, but we must declare this to
// comply with CosmWasm 0.9 API
///////////////////////////////////////////////////////////////////////

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> StdResult<MigrateResponse> {
    Err(StdError::generic_err("You can not migrate this contract"))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    use super::*;
    use crate::msg::{HandleMsg, InitMsg};

    #[test]
    fn init_test() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg::Init {
            seed_phrase: "test".to_string(),
        };
        let env = mock_env("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg);
        println!("{:?}", res.unwrap());
    }

    #[test]
    fn addition_test() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg::Init {
            seed_phrase: "test".to_string(),
        };
        let env = mock_env("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg);
        println!("{:?}", res.unwrap());

        let msg = HandleMsg::NewKey {
            key_seed: "test".to_string(),
            passphrase: "test".to_string(),
        };
        let env = mock_env("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = handle(&mut deps, env, msg);
        println!("{:?}", res.unwrap());
    }
}
