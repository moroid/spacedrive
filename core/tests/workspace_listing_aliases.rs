use std::time::Duration;

use sd_core::{
	config::AppConfig,
	domain::addressing::SdPath,
	infra::{api::SessionContext, query::LibraryQuery},
	ops::files::query::DirectoryListingQuery,
	Core,
};
use uuid::Uuid;

#[tokio::test]
async fn local_device_aliases_list_the_same_registered_folder() {
	let data = tempfile::tempdir().expect("data");
	let files = tempfile::tempdir().expect("files");
	let root = files.path().canonicalize().expect("root");
	std::fs::write(root.join("alpha.txt"), b"workspace fixture").expect("fixture file");
	let mut config = AppConfig::default_with_dir(data.path().to_path_buf());
	config.services.networking_enabled = false;
	config.services.volume_monitoring_enabled = false;
	config.services.fs_watcher_enabled = false;
	config.services.statistics_listener_enabled = false;
	let core = Core::new_with_config(data.path().to_path_buf(), Some(config), None)
		.await
		.expect("isolated core");
	core.context.startup_indexing.skip_default_sources();
	let library = core
		.libraries
		.create_library("Workspace alias test", None, core.context.clone())
		.await
		.expect("library");
	core.context
		.volume_index()
		.register_source_in(Some(library.id()), &root, None)
		.await
		.expect("registered source");
	let device_id = core.device.device_id().expect("device id");
	let session = SessionContext::device_session(device_id, "workspace test".to_string())
		.with_library(library.id());
	let canonical = sd_core::device::get_current_device_slug();
	let expected_id = tokio::time::timeout(Duration::from_secs(20), async {
		loop {
			let output = DirectoryListingQuery::new(SdPath::physical(canonical.clone(), &root))
				.execute(core.context.clone(), session.clone())
				.await
				.expect("canonical listing");
			if output.total_count == 1 {
				assert_eq!(output.files[0].name, "alpha");
				break output.files[0].id;
			}
			tokio::time::sleep(Duration::from_millis(25)).await;
		}
	})
	.await
	.expect("browse walk completed");
	for alias in ["local".to_string(), canonical, device_id.to_string()] {
		let output = DirectoryListingQuery::new(SdPath::physical(alias.clone(), &root))
			.execute(core.context.clone(), session.clone())
			.await
			.expect("alias listing");
		assert_eq!(output.total_count, 1, "alias {alias}");
		assert_eq!(
			output.files[0].id, expected_id,
			"same file identity for {alias}"
		);
	}
	let remote = DirectoryListingQuery::new(SdPath::physical(Uuid::new_v4().to_string(), &root))
		.execute(core.context.clone(), session)
		.await
		.expect("remote listing");
	assert_eq!(
		remote.total_count, 0,
		"another device must not read the local filesystem"
	);
	core.shutdown().await.expect("shutdown");
}
