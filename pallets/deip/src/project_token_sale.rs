use crate::*;

/// Contains information about tokens of the project
#[derive(Encode, Decode, Default)]
pub struct TokenInfo {
    pub total: u64,
    pub reserved: u64,
}

impl<T: Config> Module<T> {
    pub(super) fn process_project_token_sales() {
        let now = pallet_timestamp::Module::<T>::get();

        let token_sales_by_end_time = ProjectTokenSaleEndTimes::<T>::get();
        for (_, sale_id) in token_sales_by_end_time {
            let sale = ProjectTokenSaleMap::<T>::get(sale_id);
            if sale.end_time <= now && matches!(sale.status, ProjectTokenSaleStatus::Active) {
                if sale.total_amount < sale.soft_cap {
                    Self::update_status(sale, ProjectTokenSaleStatus::Expired);
                    Self::refund_project_token_sale(sale_id);
                } else if sale.total_amount >= sale.soft_cap {
                    Self::update_status(sale, ProjectTokenSaleStatus::Finished);
                    Self::finish_project_token_sale(sale_id);
                }
            } else if sale.end_time > now {
                if now >= sale.start_time && matches!(sale.status, ProjectTokenSaleStatus::Inactive) {
                    Self::update_status(sale, ProjectTokenSaleStatus::Active);
                }
            }
        }
    }

    fn update_status(sale: ProjectTokenSaleOf<T>, new_status: ProjectTokenSaleStatus) {
        ProjectTokenSaleMap::<T>::mutate_exists(sale.external_id, |maybe_sale| -> () {
            let sale = maybe_sale.as_mut().expect("we keep collections in sync");
            sale.status = new_status;
        });

        let mut token_sales = ProjectTokenSales::get();
        match new_status {
            ProjectTokenSaleStatus::Finished | ProjectTokenSaleStatus::Expired | ProjectTokenSaleStatus::Active => {
                let old_index = token_sales.binary_search_by_key(&(sale.project_id, sale.status), |&(p, t, _)| (p, t)).expect("we keep collections in sync");
                token_sales.remove(old_index);

                let index = match token_sales.binary_search_by_key(&(sale.project_id, new_status), |&(p, t, _)| (p, t)) {
                    Ok(i) => i,
                    Err(i) => i,
                };

                token_sales.insert(index, (sale.project_id, new_status, sale.external_id));
                ProjectTokenSales::put(token_sales);
            },
            _ => (),
        }
    }

    fn refund_project_token_sale(id: ProjectTokenSaleId) {
        // unimplemented!();
    }

    fn finish_project_token_sale(id: ProjectTokenSaleId) {
        // unimplemented!();
    }
}
