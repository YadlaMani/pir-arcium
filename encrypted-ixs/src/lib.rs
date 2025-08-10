use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;
    
    pub struct QueryInput {
        query_id: u64,
    }
    
    #[instruction]
    pub fn pir_query(input_ctxt: Enc<Shared, QueryInput>) -> Enc<Shared, u128> {
        let input = input_ctxt.to_arcis();
        
       
        let database_ids = [1u64, 2u64, 3u64, 4u64, 5u64];
        let database_values = [100u128, 200u128, 300u128, 400u128, 500u128];
        
        let mut result: u128 = 0;
        
       
        for i in 0..database_ids.len() {
            if database_ids[i] == input.query_id {
                result = database_values[i];
            }
        }
        
        input_ctxt.owner.from_arcis(result)
    }
}