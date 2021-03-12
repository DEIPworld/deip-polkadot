use crate::*;
use crate::{mock::*};
use sp_core::H256;
use frame_support::{assert_ok, assert_noop};

const DEFAULT_ACCOUNT_ID: <Test as system::Config>::AccountId = 123;

fn create_ok_project(maybe_account_id: Option<<Test as system::Config>::AccountId>) 
	-> (ProjectOf<Test>, ProjectId, Domain, <Test as system::Config>::AccountId, ) {
	let domain = Domain::random();
	let account_id: <Test as system::Config>::AccountId = maybe_account_id.unwrap_or(DEFAULT_ACCOUNT_ID);
	let project_id = ProjectId::random();
	
	assert_ok!(Deip::add_domain(Origin::signed(account_id), domain.clone()));

	let project = ProjectOf::<Test> {
		is_private: false,
		external_id: project_id,
		team_id: account_id,
		description: H256::random(),
		domains: vec![domain],
		members: vec![account_id],
	};
	
	assert_ok!(Deip::create_project(Origin::signed(account_id), project.clone()));

	(project, project_id, domain, account_id)
}

#[test]
fn add_domain() {
	new_test_ext().execute_with(|| {
		let domain = Domain::random();
		// Dispatch a signed add domian extrinsic.
		assert_ok!(Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), domain.clone()));
		
		// Read pallet storage and assert an expected result.
		assert_eq!(Deip::domain_count(), 1);
		assert!(
			<Domains>::contains_key(domain),
			"Domains did not contain domain, value was `{}`",
            domain
		);
	});
}

#[test]
fn cant_add_duplicate_domain() {
	new_test_ext().execute_with(|| {
		let domain = Domain::random();
		
		assert_ok!(Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), domain.clone()));

		assert_noop!(
			Deip::add_domain(Origin::signed(DEFAULT_ACCOUNT_ID), domain.clone()),
			Error::<Test>::DomainAlreadyExists
		);
	})
}

#[test]
fn add_project() {
	new_test_ext().execute_with(|| {
		let (project ,project_id, ..) = create_ok_project(None);

		
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
		let domain = Domain::random();
		let account_id = DEFAULT_ACCOUNT_ID;
		
		let project = Project {
			is_private: false,
			external_id: ProjectId::random(),
			team_id: account_id,
			description: H256::random(),
			domains: vec![domain],
			members: vec![account_id],
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
		let (project, ..) = create_ok_project(None);

		assert_noop!(
			Deip::create_project(Origin::signed(DEFAULT_ACCOUNT_ID), project.clone()),
			Error::<Test>::ProjectAlreadyExists
		);

	})
}


#[test]
fn update_project() {
	new_test_ext().execute_with(|| {
		let (_ ,project_id, ..) = create_ok_project(None);

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

		let (_ ,project_id, ..) = create_ok_project(Some(account_id));

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
		let (_ ,project_id, ..) = create_ok_project(None);
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
		let (_ ,project_id, ..) = create_ok_project(None);
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

		let project_content_with_reference = ProjectContentOf::<Test> {
			references: Some(vec![project_content_id]),
			external_id: ProjectContentId::random(),
			..project_content.clone()
		};

		assert_ok!(Deip::create_project_content(Origin::signed(DEFAULT_ACCOUNT_ID), project_content_with_reference));

	})
}

#[test]
fn cant_add_duplicated_project_content() {
	new_test_ext().execute_with(|| {
		let (_ ,project_id, ..) = create_ok_project(None);

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
		let (_ ,project_id, ..) = create_ok_project(None);
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
		let (_ ,project_id, ..) = create_ok_project(None);

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
		let (_ ,project_id, ..) = create_ok_project(None);
		
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
