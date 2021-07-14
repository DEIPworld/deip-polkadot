use crate::*;
use crate::{mock::*};
use sp_core::H256;
use frame_support::{assert_ok, assert_noop};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_ACCOUNT_ID: <Test as system::Config>::AccountId = 123;
const DAY_IN_MILLIS: u64 = 86400000;

fn create_ok_project(maybe_account_id: Option<<Test as system::Config>::AccountId>) 
	-> (ProjectId, ProjectOf<Test>, DomainId, <Test as system::Config>::AccountId, ) {
	let domain_id = DomainId::random();
	let account_id: <Test as system::Config>::AccountId = maybe_account_id.unwrap_or(DEFAULT_ACCOUNT_ID);
	let project_id = ProjectId::random();
	
	assert_ok!(Deip::add_domain(Origin::signed(account_id), Domain { external_id: domain_id.clone() }));

	let project = ProjectOf::<Test> {
		is_private: false,
		external_id: project_id,
		team_id: account_id,
		description: H256::random(),
		domains: vec![domain_id],
	};
	
	assert_ok!(Deip::create_project(Origin::signed(account_id), project.clone()));

	(project_id, project, domain_id, account_id)
}

fn create_ok_nda() -> (NdaId, NdaOf<Test>) {
	let (project_id, ..) = create_ok_project(None);
	let project_nda_id =  NdaId::random();
	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards :)").as_millis() as u64;
	
	let end_date = now + DAY_IN_MILLIS;
	let contract_hash = H256::random();
	let maybe_start_date = None;
	let parties = vec![DEFAULT_ACCOUNT_ID];
	let projects = vec![project_id];
	

	assert_ok!(
		Deip::create_project_nda(
			Origin::signed(DEFAULT_ACCOUNT_ID), 
			project_nda_id, 
			end_date, 
			contract_hash, 
			maybe_start_date,
			parties.clone(),
			projects.clone()
		)
	);

	let expected_nda  = Nda {
		contract_creator: DEFAULT_ACCOUNT_ID,
		external_id: project_nda_id,
		end_date,
		start_date: maybe_start_date,
		contract_hash,
		parties,
		projects
	};

	(project_nda_id, expected_nda)
		
}

fn create_ok_nda_content_access_request(project_nda_id: NdaId) -> (NdaAccessRequestId, NdaAccessRequestOf<Test>) {
	let access_request_id = NdaAccessRequestId::random();
	let encrypted_payload_hash = H256::random();
	let encrypted_payload_iv = vec![1, 2, 3];

	assert_ok!(
		Deip::create_nda_content_access_request(
			Origin::signed(DEFAULT_ACCOUNT_ID), 
			access_request_id, 
			project_nda_id,
			encrypted_payload_hash, 
			encrypted_payload_iv.clone()
		)
	);

	let expected_nda_request = NdaAccessRequest {
		external_id: access_request_id,
		nda_external_id: project_nda_id, 
		requester: DEFAULT_ACCOUNT_ID,
		encrypted_payload_hash,
		encrypted_payload_iv,
		status: NdaAccessRequestStatus::Pending,
		grantor: None,
		encrypted_payload_encryption_key: None,
		proof_of_encrypted_payload_encryption_key: None,
	};

	(access_request_id, expected_nda_request)
}

#[test]
fn add_domain() {
	new_test_ext().execute_with(|| {
		let domain_id = DomainId::random();
		// Dispatch a signed add domian extrinsic.
		assert_ok!(Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), Domain { external_id: domain_id.clone() }));
		
		// Read pallet storage and assert an expected result.
		assert_eq!(Deip::domain_count(), 1);
		assert!(
			<Domains>::contains_key(domain_id),
			"Domains did not contain domain, value was `{}`",
            domain_id
		);
	});
}

#[test]
fn cant_add_duplicate_domain() {
	new_test_ext().execute_with(|| {
		let domain_id = DomainId::random();
		
		assert_ok!(Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), Domain { external_id: domain_id.clone() }));

		assert_noop!(
			Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), Domain { external_id: domain_id.clone() }),
			Error::<Test>::DomainAlreadyExists
		);
	})
}

#[test]
fn add_project() {
	new_test_ext().execute_with(|| {
		let (project_id ,project, ..) = create_ok_project(None);

		
		// TODO Add event check
		// let expected_event = mock::Event::pallet_deip(crate::Event::ProjectCreated(account_id, project)::<<Test as system::Config>::AccountId, ProjectOf<Test>>);

		// assert_eq!(
		// 	System::events()[0].event,
		// 	expected_event,
		// );

		let projects = Projects::<Test>::get();
		let project_stored = ProjectMap::<Test>::get(project_id);

		assert!(
			<ProjectMap<Test>>::contains_key(project_id),
			"Project Map did not contain the project, value was `{}`",
            project_id
		);

		assert_eq!(project, project_stored);

		assert!(
			projects.binary_search_by_key(&project_id, |&(external_id, ..)| external_id).is_ok(),
			"Projects did not contain project, value was `{}`",
            project_id
		);

	})
}

#[test]
fn cant_add_project_with_non_exixsted_domain() {
	new_test_ext().execute_with(|| {
		let domain = DomainId::random();
		let account_id = DEFAULT_ACCOUNT_ID;
		
		let project = Project {
			is_private: false,
			external_id: ProjectId::random(),
			team_id: account_id,
			description: H256::random(),
			domains: vec![domain],
		};
		
		assert_noop!(
			Deip::create_project(Origin::signed(DEFAULT_ACCOUNT_ID), project.clone()),
			Error::<Test>::DomainNotExists
		);
	})
}

#[test]
fn cant_add_duplicated_project() {
	new_test_ext().execute_with(|| {
		let (_, project, ..) = create_ok_project(None);

		assert_noop!(
			Deip::create_project(Origin::signed(DEFAULT_ACCOUNT_ID), project.clone()),
			Error::<Test>::ProjectAlreadyExists
		);

	})
}


#[test]
fn update_project() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);

		let new_description = H256::random();
		let new_members = vec![1,2];

		assert_ok!(Deip::update_project(Origin::signed(DEFAULT_ACCOUNT_ID), project_id, Some(new_description), Some(true), Some(new_members.clone())));


		let project_stored = ProjectMap::<Test>::get(project_id);

		assert_eq!(project_stored.description, new_description);
		assert_eq!(project_stored.is_private, true);
		assert_eq!(project_stored.members, new_members);


	})
}


#[test]
fn cant_update_project_not_belonged_to_your_signature() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 2;
		let wrong_account_id = 1;

		let (project_id, ..) = create_ok_project(Some(account_id));

		let new_description = H256::random();
		let new_members = vec![1,2];

		assert_noop!(
			Deip::update_project(Origin::signed(wrong_account_id), project_id, Some(new_description), Some(true), Some(new_members.clone())),
			Error::<Test>::NoPermission
		);
	})
}

#[test]
fn cant_update_not_existed_project() {
	new_test_ext().execute_with(|| {
		let project_id = ProjectId::random();

		assert_noop!(
			Deip::update_project(Origin::signed(DEFAULT_ACCOUNT_ID), project_id, None, None, None),
			Error::<Test>::NoSuchProject
		);
	})
}


#[test]
fn create_project_content() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let project_content_id =  ProjectContentId::random();

		let project_content = ProjectContentOf::<Test> {
			external_id: project_content_id,
			project_external_id: project_id,
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None
			
		};
		

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content.clone()));

		let project_content_list = ProjectsContent::<Test>::get();
		let project_content_stored = ProjectContentMap::<Test>::get(project_id, project_content_id);

		assert!(
			<ProjectContentMap<Test>>::contains_key(project_id, project_content_id),
			"Project Content Map did not contain key, value was `{}{}`",
            project_id,
			project_content_id

		);

		assert_eq!(project_content, project_content_stored);

		assert!(
			project_content_list.binary_search_by_key(&project_content_id, |&(external_id, ..)| external_id).is_ok(),
			"Projects Contntent List did not contain the content, value was `{}`",
            project_content_id
		);

	})
}

#[test]
fn create_project_content_with_references() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let project_content_id = ProjectContentId::random();

		let project_content = ProjectContentOf::<Test> {
			external_id: project_content_id,
			project_external_id: project_id,
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None	
		};
		

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content.clone()));

		let project_content_with_reference_id =  ProjectContentId::random();

		let project_content_with_reference = ProjectContentOf::<Test> {
			references: Some(vec![project_content_id]),
			external_id: project_content_with_reference_id,
			..project_content.clone()
		};

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content_with_reference.clone()));

		let project_content_list = ProjectsContent::<Test>::get();
		let project_content_stored = ProjectContentMap::<Test>::get(project_id, project_content_with_reference_id);

		assert!(
			<ProjectContentMap<Test>>::contains_key(project_id, project_content_with_reference_id),
			"Project Content Map did not contain key, value was `{}{}`",
            project_id,
			project_content_with_reference_id

		);

		assert_eq!(project_content_with_reference, project_content_stored);

		assert!(
			project_content_list.binary_search_by_key(&project_content_with_reference_id, |&(external_id, ..)| external_id).is_ok(),
			"Projects Contntent List did not contain the content, value was `{}`",
            project_content_with_reference_id
		);

	})
}

#[test]
fn cant_add_duplicated_project_content() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);

		let project_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			project_external_id: project_id,
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None
			
		};
		

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content.clone()));

		assert_noop!(
			Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content),
			Error::<Test>::ProjectContentAlreadyExists
		);

	})
}


#[test]
fn cant_add_project_content_with_wrong_project_reference() {
	new_test_ext().execute_with(|| {
		let project_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			project_external_id: ProjectId::random(),
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None
			
		};

		assert_noop!(
			Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content),
			Error::<Test>::NoSuchProject
		);

	})
}

#[test]
fn cant_add_project_content_to_incorrect_team() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let wrong_account_id = 234;

		let project_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			project_external_id: project_id,
			team_id: wrong_account_id,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None
		};
		

		assert_noop!(
			Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content),
			Error::<Test>::ProjectNotBelongToTeam
		);

	})
}

#[test]
fn cant_add_project_content_to_finished_project() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);

		let project_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			project_external_id: project_id,
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::FinalResult,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: None
			
		};

		let another_proeject_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			content_type: ProjectContentType::MilestoneCode,
			..project_content.clone()
		};
		

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content));

		assert_noop!(
			Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), another_proeject_content),
			Error::<Test>::ProjectAlreadyFinished
		);
	})
}


#[test]
fn cant_add_project_content_with_wrong_references() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		
		let project_content = ProjectContentOf::<Test> {
			external_id: ProjectContentId::random(),
			project_external_id: project_id,
			team_id: DEFAULT_ACCOUNT_ID,
			content_type: ProjectContentType::Announcement,
			description: H256::random(),
			content: H256::random(),
			authors: vec![DEFAULT_ACCOUNT_ID],
			references: Some(vec![ProjectContentId::random()])
			
		};

		assert_noop!(
			Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content),
			Error::<Test>::NoSuchReference
		);

	})
}


#[test]
fn create_project_nda() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, expected_nda) = create_ok_nda();

		let nda_list = Ndas::<Test>::get();
		let nda_stored = NdaMap::<Test>::get(project_nda_id);

		assert!(
			<NdaMap<Test>>::contains_key(project_nda_id),
			"NDA Map did not contain key, value was `{}`",
			project_nda_id

		);

		assert_eq!(expected_nda, nda_stored);

		assert!(
			nda_list.binary_search_by_key(&project_nda_id, |&(external_id, ..)| external_id).is_ok(),
			"NDA List did not contain the NDA, value was `{}`",
            project_nda_id
		);

	})
}

#[test]
fn cant_create_project_nda_ends_in_past() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let project_nda_id =  NdaId::random();		
		let end_date = 0;
		
		let contract_hash = H256::random();
		let maybe_start_date = None;
		let parties = vec![DEFAULT_ACCOUNT_ID];
		let projects = vec![project_id];
		

		assert_noop!(
			Deip::create_project_nda(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				project_nda_id, 
				end_date, 
				contract_hash, 
				maybe_start_date,
				parties.clone(),
				projects.clone()
			),
			Error::<Test>::NdaEndDateMustBeLaterCurrentMoment
		);

	})
}

#[test]
fn cant_create_project_nda_with_start_date_greater_end_date() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let project_nda_id =  NdaId::random();		
		
		let end_date = 1;
		let maybe_start_date = Some(3);
		
		let contract_hash = H256::random();
		
		let parties = vec![DEFAULT_ACCOUNT_ID];
		let projects = vec![project_id];
		

		assert_noop!(
			Deip::create_project_nda(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				project_nda_id, 
				end_date, 
				contract_hash, 
				maybe_start_date,
				parties.clone(),
				projects.clone()
			),
			Error::<Test>::NdaStartDateMustBeLessThanEndDate
		);

	})
}


#[test]
fn cant_create_project_nda_with_non_existed_project() {
	new_test_ext().execute_with(|| {
		let project_id = ProjectId::random();
		let project_nda_id =  NdaId::random();		
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards :)").as_millis() as u64;
	
		let end_date = now + DAY_IN_MILLIS;
		
		let contract_hash = H256::random();
		let maybe_start_date = None;
		let parties = vec![DEFAULT_ACCOUNT_ID];
		let projects = vec![project_id];
		

		assert_noop!(
			Deip::create_project_nda(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				project_nda_id, 
				end_date, 
				contract_hash, 
				maybe_start_date,
				parties.clone(),
				projects.clone()
			),
			Error::<Test>::NoSuchProject
		);

	})
}

#[test]
fn cant_create_project_nda_with_not_correct_parties() {
	new_test_ext().execute_with(|| {
		let (project_id, ..) = create_ok_project(None);
		let project_nda_id =  NdaId::random();		
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards :)").as_millis() as u64;
	
		let end_date = now + DAY_IN_MILLIS;

		let wrong_account_id = 4;
		
		let contract_hash = H256::random();
		let maybe_start_date = None;
		let parties = vec![wrong_account_id];
		let projects = vec![project_id];
		

		assert_noop!(
			Deip::create_project_nda(
				Origin::signed(wrong_account_id), 
				project_nda_id, 
				end_date, 
				contract_hash, 
				maybe_start_date,
				parties.clone(),
				projects.clone()
			),
			Error::<Test>::TeamOfAllProjectsMustSpecifiedAsParty
		);

	})
}

#[test]
fn cant_create_duplicated_project_nda() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();
		
		let (project_id, ..) = create_ok_project(None);	
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.expect("Time went backwards :)").as_millis() as u64;
	
		let end_date = now + DAY_IN_MILLIS;
		
		let contract_hash = H256::random();
		let maybe_start_date = None;
		let parties = vec![DEFAULT_ACCOUNT_ID];
		let projects = vec![project_id];
		

		assert_noop!(
			Deip::create_project_nda(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				project_nda_id, 
				end_date, 
				contract_hash, 
				maybe_start_date,
				parties.clone(),
				projects.clone()
			),
			Error::<Test>::NdaAlreadyExists
		);

	})
}

#[test]
fn create_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();

		let (access_request_id, expected_nda_request) = create_ok_nda_content_access_request(project_nda_id);

		let nda_list = NdaAccessRequests::<Test>::get();
		let nda_stored = NdaAccessRequestMap::<Test>::get(access_request_id);

		assert!(
			<NdaAccessRequestMap<Test>>::contains_key(access_request_id),
			"NDA request Map did not contain key, value was `{}`",
			access_request_id

		);

		assert_eq!(expected_nda_request, nda_stored);

		assert!(
			nda_list.binary_search_by_key(&access_request_id, |&(external_id, ..)| external_id).is_ok(),
			"NDA request List did not contain the NDA request, value was `{}`",
            access_request_id
		);

	})
}


#[test]
fn cant_create_nda_content_access_with_non_existed_nda() {
	new_test_ext().execute_with(|| {
		let project_nda_id = NdaId::random();

		let access_request_id = NdaAccessRequestId::random();
		let encrypted_payload_hash = H256::random();
		let encrypted_payload_iv = vec![1, 2, 3];

		assert_noop!(
			Deip::create_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id, 
				project_nda_id,
				encrypted_payload_hash, 
				encrypted_payload_iv.clone()
			),
			Error::<Test>::NoSuchNda
		);

	})
}

#[test]
fn cant_create_duplicated_nda_content_access() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();
		let (access_request_id, expected_nda_request) = create_ok_nda_content_access_request(project_nda_id);

		assert_noop!(
			Deip::create_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id, 
				project_nda_id,
				expected_nda_request.encrypted_payload_hash, 
				expected_nda_request.encrypted_payload_iv
			),
			Error::<Test>::NdaAccessRequestAlreadyExists
		);

	})
}


#[test]
fn fulfill_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();

		let (access_request_id, nda_request) = create_ok_nda_content_access_request(project_nda_id);

		let encrypted_payload_encryption_key = vec![1,3,4,2];
		let proof_of_encrypted_payload_encryption_key = vec![3,4,5,6];

		assert_ok!(
			Deip::fulfill_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
				encrypted_payload_encryption_key.clone(), 
				proof_of_encrypted_payload_encryption_key.clone()
			)
		);

		let nda_stored = NdaAccessRequestMap::<Test>::get(access_request_id);

		let expected_nda_request = NdaAccessRequest {
			status: NdaAccessRequestStatus::Fulfilled,
			grantor: Some(DEFAULT_ACCOUNT_ID),
			encrypted_payload_encryption_key: Some(encrypted_payload_encryption_key),
			proof_of_encrypted_payload_encryption_key: Some(proof_of_encrypted_payload_encryption_key),
			..nda_request
		};

		assert_eq!(expected_nda_request, nda_stored);

	})
}


#[test]
fn cant_fulfill_not_existed_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let access_request_id = NdaAccessRequestId::random();

		let encrypted_payload_encryption_key = vec![1,3,4,2];
		let proof_of_encrypted_payload_encryption_key = vec![3,4,5,6];

		assert_noop!(
			Deip::fulfill_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
				encrypted_payload_encryption_key.clone(), 
				proof_of_encrypted_payload_encryption_key.clone()
			),
			Error::<Test>::NoSuchNdaAccessRequest
		);

	})
}

#[test]
fn cant_fulfill_finalized_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();

		let (access_request_id, ..) = create_ok_nda_content_access_request(project_nda_id);

		let encrypted_payload_encryption_key = vec![1,3,4,2];
		let proof_of_encrypted_payload_encryption_key = vec![3,4,5,6];

		assert_ok!(
			Deip::fulfill_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
				encrypted_payload_encryption_key.clone(), 
				proof_of_encrypted_payload_encryption_key.clone()
			)
		);

		assert_noop!(
			Deip::fulfill_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
				encrypted_payload_encryption_key.clone(), 
				proof_of_encrypted_payload_encryption_key.clone()
			),
			Error::<Test>::NdaAccessRequestAlreadyFinalized
		);

	})
}

#[test]
fn reject_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();

		let (access_request_id, nda_request) = create_ok_nda_content_access_request(project_nda_id);

		assert_ok!(
			Deip::reject_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
			)
		);

		let nda_stored = NdaAccessRequestMap::<Test>::get(access_request_id);

		let expected_nda_request = NdaAccessRequest {
			status: NdaAccessRequestStatus::Rejected,
			..nda_request
		};

		assert_eq!(expected_nda_request, nda_stored);

	})
}


#[test]
fn cant_reject_not_existed_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let access_request_id = NdaAccessRequestId::random();

		assert_noop!(
			Deip::reject_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
			),
			Error::<Test>::NoSuchNdaAccessRequest
		);

	})
}

#[test]
fn cant_reject_finalized_nda_content_access_request() {
	new_test_ext().execute_with(|| {
		let (project_nda_id, ..) = create_ok_nda();

		let (access_request_id, ..) = create_ok_nda_content_access_request(project_nda_id);

		assert_ok!(
			Deip::reject_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
			)
		);

		assert_noop!(
			Deip::reject_nda_content_access_request(
				Origin::signed(DEFAULT_ACCOUNT_ID), 
				access_request_id.clone(), 
			),
			Error::<Test>::NdaAccessRequestAlreadyFinalized
		);

	})
}
