//! Explicit launch settings keep native workspace tests off existing libraries.

use std::{ffi::OsString, net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceTestLaunch {
	pub data_dir: PathBuf,
	pub daemon_addr: SocketAddr,
}

impl WorkspaceTestLaunch {
	pub fn webview_data_dir(&self) -> PathBuf {
		self.data_dir.join("webview")
	}

	pub fn from_values(
		data_dir: Option<OsString>,
		daemon_addr: Option<OsString>,
	) -> anyhow::Result<Option<Self>> {
		let (data_dir, daemon_addr) = match (data_dir, daemon_addr) {
			(None, None) => return Ok(None),
			(Some(data_dir), Some(daemon_addr)) => (PathBuf::from(data_dir), daemon_addr),
			_ => anyhow::bail!("Workspace tests require both SD_WORKSPACE_TEST_DATA_DIR and SD_WORKSPACE_TEST_DAEMON_ADDR"),
		};
		anyhow::ensure!(
			data_dir.is_absolute(),
			"Workspace test data directory must be absolute"
		);
		let daemon_addr: SocketAddr = daemon_addr
			.to_str()
			.ok_or_else(|| anyhow::anyhow!("Workspace test daemon address must be Unicode"))?
			.parse()?;
		anyhow::ensure!(
			daemon_addr.ip().is_loopback(),
			"Workspace test daemon must use loopback"
		);
		anyhow::ensure!(
			daemon_addr.port() != 0 && daemon_addr.port() != 6969,
			"Workspace tests must use a dedicated daemon port"
		);
		Ok(Some(Self {
			data_dir,
			daemon_addr,
		}))
	}
}

pub fn workspace_test_launch() -> anyhow::Result<Option<WorkspaceTestLaunch>> {
	WorkspaceTestLaunch::from_values(
		std::env::var_os("SD_WORKSPACE_TEST_DATA_DIR"),
		std::env::var_os("SD_WORKSPACE_TEST_DAEMON_ADDR"),
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn absolute_dir() -> OsString {
		std::env::temp_dir()
			.join("workspace-test-launch")
			.into_os_string()
	}

	#[test]
	fn ordinary_launch_keeps_existing_defaults() {
		assert_eq!(WorkspaceTestLaunch::from_values(None, None).unwrap(), None);
	}

	#[test]
	fn isolated_launch_requires_both_settings() {
		assert!(WorkspaceTestLaunch::from_values(Some(absolute_dir()), None).is_err());
		assert!(WorkspaceTestLaunch::from_values(None, Some("127.0.0.1:7070".into())).is_err());
	}

	#[test]
	fn isolated_launch_rejects_unsafe_or_ambiguous_destinations() {
		assert!(WorkspaceTestLaunch::from_values(
			Some("relative".into()),
			Some("127.0.0.1:7070".into())
		)
		.is_err());
		for address in [
			"0.0.0.0:7070",
			"192.0.2.1:7070",
			"127.0.0.1:6969",
			"127.0.0.1:0",
			"localhost:7070",
			"",
		] {
			assert!(
				WorkspaceTestLaunch::from_values(Some(absolute_dir()), Some(address.into()))
					.is_err(),
				"{address}"
			);
		}
	}

	#[test]
	fn isolated_launch_preserves_unicode_directory_and_loopback_endpoint() {
		let path = PathBuf::from(absolute_dir()).join("日本語 テスト");
		let result = WorkspaceTestLaunch::from_values(
			Some(path.clone().into_os_string()),
			Some("127.0.0.1:7070".into()),
		)
		.unwrap()
		.unwrap();
		assert_eq!(result.data_dir, path);
		assert_eq!(result.webview_data_dir(), path.join("webview"));
		assert_eq!(result.daemon_addr.to_string(), "127.0.0.1:7070");
	}
}
