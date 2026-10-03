//! GENERATED FILE — DO NOT EDIT BY HAND.
//!
//! The entire service command tree, compiled from the committed
//! Discovery index (`src/discovery/*.json`) by
//! `scripts/generate-commands.mjs`. The index is refreshed daily by
//! `.github/workflows/discovery.yml`; after any index change,
//! regenerate with `node scripts/generate-commands.mjs` (or run it
//! with `--check`, which exits 1 when this file is stale).
//!
//! Generated at: 2026-10-03T09:51:48.428Z — the index manifest's own timestamp,
//! so regeneration is byte-identical until the index actually changes.
//!
//! Shape:
//!   * one top-level subcommand per service;
//!   * nested resources nest as subcommands (gmail -> users -> messages),
//!     verbatim id segments, never flattened;
//!   * one leaf per method. A leaf's clap name IS its full dotted
//!     method id (`gmail.users.messages.list`), with a visible alias of
//!     the bare method name, so dispatch resolves the id without
//!     reconstructing paths and `grr schema` self-documents every
//!     callable id;
//!   * every leaf flag set = one typed flag per Discovery parameter
//!     (camelCase -> kebab-case; integer -> i64; boolean -> presence
//!     flag; repeated -> repeatable; enum -> possible values; required
//!     -> required) plus the escape hatches `--params`, `--body-file`,
//!     `--query`, `--dry-run`, `-f/--format`.
//!
//! Honest limitation: request bodies are NOT typed. Discovery's request
//! schemas are not part of the distilled index, so POST/PATCH/PUT
//! bodies pass through `--params` / `--body-file` verbatim.
//!
//! Numbers: 14 services, 401 methods, 401 leaves, 96 resource groups, 994 typed flags.

/// When the generator last ran, taken from the Discovery index
/// manifest's own timestamp so it only moves when the index moves.
pub const GENERATED_AT: &str = "2026-10-03T09:51:48.428Z";

#[rustfmt::skip] // mechanical output; formatting it would churn every diff
pub mod tree {
    use clap::{Arg, ArgAction, Command};
    use crate::output::OutputFormat;

    /// The escape hatches every generated leaf shares with `grr api call`.
    /// Dispatch in gen_dispatch.rs reads them by these exact ids.
    fn escape_hatch_args() -> Vec<Arg> {
        vec![
            Arg::new("params").long("params").value_name("JSON")
                .help("Method parameters as a JSON object. Merged before the typed flags, so a typed flag always wins on conflict"),
            Arg::new("body-file").long("body-file").value_name("PATH|-")
                .help("Read the request body from a file ('-' = stdin) and send it verbatim (POST/PATCH/PUT)"),
            Arg::new("query").long("query").value_name("KEY=VALUE").action(ArgAction::Append)
                .help("Extra query pair for parameters Discovery does not document (e.g. alt=json); repeatable"),
            Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue)
                .help("Print the request that would be sent, without sending it"),
            Arg::new("format").short('f').long("format").value_name("FORMAT")
                .value_parser(clap::value_parser!(OutputFormat)).default_value("json")
                .help("Output format"),
        ]
    }
    /// All 14 service commands, in `discovery::services()` order.
    pub fn commands() -> Vec<Command> {
        vec![
            service_analyticsadmin(),
            service_analyticsdata(),
            service_calendar(),
            service_chat(),
            service_docs(),
            service_drive(),
            service_forms(),
            service_gmail(),
            service_people(),
            service_script(),
            service_searchconsole(),
            service_sheets(),
            service_slides(),
            service_tasks(),
        ]
    }

    // analyticsadmin.accounts.delete
    fn leaf_analyticsadmin_accounts_delete() -> Command {
        Command::new("analyticsadmin.accounts.delete")
            .visible_alias("delete")
            .about("Marks target Account as soft-deleted (ie: \"trashed\") and returns it. This API does not have a...")
            .long_about("Marks target Account as soft-deleted (ie: \"trashed\") and returns it. This API does not have a method to restore soft-deleted accounts. However, they can be restored using the Trash Can UI. If the accounts are not restored before the expiration time, the account and all child resources (eg: Propertie")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the Account to soft-delete. Format: accounts/{account} Example: \"accounts/100\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.get
    fn leaf_analyticsadmin_accounts_get() -> Command {
        Command::new("analyticsadmin.accounts.get")
            .visible_alias("get")
            .about("Lookup for a single Account.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the account to lookup. Format: accounts/{account} Example: \"accounts/100\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.getDataSharingSettings
    fn leaf_analyticsadmin_accounts_get_data_sharing_settings() -> Command {
        Command::new("analyticsadmin.accounts.getDataSharingSettings")
            .visible_alias("getDataSharingSettings")
            .about("Get data sharing settings on an account. Data sharing settings are singletons.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the settings to lookup. Format: accounts/{account}/dataSharingSettings Example: `accounts/1000/dataSharingSettings`"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.list
    fn leaf_analyticsadmin_accounts_list() -> Command {
        Command::new("analyticsadmin.accounts.list")
            .visible_alias("list")
            .about("Returns all accounts accessible by the caller. Note that these accounts might not currently have GA...")
            .long_about("Returns all accounts accessible by the caller. Note that these accounts might not currently have GA properties. Soft-deleted (ie: \"trashed\") accounts are excluded by default. Returns an empty list if no relevant accounts are found. Note: The easiest way to retrieve a list of all properties you have")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. The service may return fewer than this value, even if there are additional pages. If unspecified, at most 5"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListAccounts` call. Provide this to retrieve the subsequent page. When paginating, all other parameters provid"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include soft-deleted (ie: \"trashed\") Accounts in the results. Accounts can be inspected to determine whether they are deleted or not."))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.patch
    fn leaf_analyticsadmin_accounts_patch() -> Command {
        Command::new("analyticsadmin.accounts.patch")
            .visible_alias("patch")
            .about("Updates an account.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of this account. Format: accounts/{account} Example: \"accounts/100\""))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (for example, \"field_to_update\"). Omitted fields will not be updated. To replace t"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.provisionAccountTicket
    fn leaf_analyticsadmin_accounts_provision_account_ticket() -> Command {
        Command::new("analyticsadmin.accounts.provisionAccountTicket")
            .visible_alias("provisionAccountTicket")
            .about("Requests a ticket for creating an account.")
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.runAccessReport
    fn leaf_analyticsadmin_accounts_run_access_report() -> Command {
        Command::new("analyticsadmin.accounts.runAccessReport")
            .visible_alias("runAccessReport")
            .about("Returns a customized report of data access records. The report provides records of each time a user...")
            .long_about("Returns a customized report of data access records. The report provides records of each time a user reads Google Analytics reporting data. Access records are retained for up to 2 years. Data Access Reports can be requested for a property. Reports may be requested for any property, but dimensions tha")
            .arg(Arg::new("entity").long("entity").value_name("ENTITY").required(true)
                .help("The Data Access Report supports requesting at the property level or account level. If requested at the account level, Data Access Reports include all access for"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts.searchChangeHistoryEvents
    fn leaf_analyticsadmin_accounts_search_change_history_events() -> Command {
        Command::new("analyticsadmin.accounts.searchChangeHistoryEvents")
            .visible_alias("searchChangeHistoryEvents")
            .about("Searches through all changes to an account or its children given the specified set of filters. Only...")
            .long_about("Searches through all changes to an account or its children given the specified set of filters. Only returns the subset of changes supported by the API. The UI may return additional changes.")
            .arg(Arg::new("account").long("account").value_name("ACCOUNT").required(true)
                .help("Required. The account resource for which to return change history resources. Format: accounts/{account} Example: `accounts/100`"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accounts
    fn group_analyticsadmin_accounts() -> Command {
        Command::new("accounts")
            .about("Methods under analyticsadmin.accounts")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_accounts_delete())
            .subcommand(leaf_analyticsadmin_accounts_get())
            .subcommand(leaf_analyticsadmin_accounts_get_data_sharing_settings())
            .subcommand(leaf_analyticsadmin_accounts_list())
            .subcommand(leaf_analyticsadmin_accounts_patch())
            .subcommand(leaf_analyticsadmin_accounts_provision_account_ticket())
            .subcommand(leaf_analyticsadmin_accounts_run_access_report())
            .subcommand(leaf_analyticsadmin_accounts_search_change_history_events())
    }

    // analyticsadmin.accountSummaries.list
    fn leaf_analyticsadmin_account_summaries_list() -> Command {
        Command::new("analyticsadmin.accountSummaries.list")
            .visible_alias("list")
            .about("Returns summaries of all accounts accessible by the caller.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of AccountSummary resources to return. The service may return fewer than this value, even if there are additional pages. If unspeci"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListAccountSummaries` call. Provide this to retrieve the subsequent page. When paginating, all other parameter"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.accountSummaries
    fn group_analyticsadmin_account_summaries() -> Command {
        Command::new("accountSummaries")
            .about("Methods under analyticsadmin.accountSummaries")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_account_summaries_list())
    }

    // analyticsadmin.properties.acknowledgeUserDataCollection
    fn leaf_analyticsadmin_properties_acknowledge_user_data_collection() -> Command {
        Command::new("analyticsadmin.properties.acknowledgeUserDataCollection")
            .visible_alias("acknowledgeUserDataCollection")
            .about("Acknowledges the terms of user data collection for the specified property. This acknowledgement...")
            .long_about("Acknowledges the terms of user data collection for the specified property. This acknowledgement must be completed (either in the Google Analytics UI or through this API) before MeasurementProtocolSecret resources may be created.")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("Required. The property for which to acknowledge user data collection."))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents.create
    fn leaf_analyticsadmin_properties_conversion_events_create() -> Command {
        Command::new("analyticsadmin.properties.conversionEvents.create")
            .visible_alias("create")
            .about("Deprecated: Use `CreateKeyEvent` instead. Creates a conversion event with the specified attributes.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the parent property where this conversion event will be created. Format: properties/123"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents.delete
    fn leaf_analyticsadmin_properties_conversion_events_delete() -> Command {
        Command::new("analyticsadmin.properties.conversionEvents.delete")
            .visible_alias("delete")
            .about("Deprecated: Use `DeleteKeyEvent` instead. Deletes a conversion event in a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the conversion event to delete. Format: properties/{property}/conversionEvents/{conversion_event} Example: \"properties/123/conver"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents.get
    fn leaf_analyticsadmin_properties_conversion_events_get() -> Command {
        Command::new("analyticsadmin.properties.conversionEvents.get")
            .visible_alias("get")
            .about("Deprecated: Use `GetKeyEvent` instead. Retrieve a single conversion event.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the conversion event to retrieve. Format: properties/{property}/conversionEvents/{conversion_event} Example: \"properties/123/conv"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents.list
    fn leaf_analyticsadmin_properties_conversion_events_list() -> Command {
        Command::new("analyticsadmin.properties.conversionEvents.list")
            .visible_alias("list")
            .about("Deprecated: Use `ListKeyEvents` instead. Returns a list of conversion events in the specified...")
            .long_about("Deprecated: Use `ListKeyEvents` instead. Returns a list of conversion events in the specified parent property. Returns an empty list if no conversion events are found.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200; (higher values will be coe"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListConversionEvents` call. Provide this to retrieve the subsequent page. When paginating, all other parameter"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the parent property. Example: 'properties/123'"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents.patch
    fn leaf_analyticsadmin_properties_conversion_events_patch() -> Command {
        Command::new("analyticsadmin.properties.conversionEvents.patch")
            .visible_alias("patch")
            .about("Deprecated: Use `UpdateKeyEvent` instead. Updates a conversion event with the specified attributes.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of this conversion event. Format: properties/{property}/conversionEvents/{conversion_event}"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (e.g., \"field_to_update\"). Omitted fields will not be updated. To replace the enti"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.conversionEvents
    fn group_analyticsadmin_properties_conversion_events() -> Command {
        Command::new("conversionEvents")
            .about("Methods under analyticsadmin.properties.conversionEvents")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_conversion_events_create())
            .subcommand(leaf_analyticsadmin_properties_conversion_events_delete())
            .subcommand(leaf_analyticsadmin_properties_conversion_events_get())
            .subcommand(leaf_analyticsadmin_properties_conversion_events_list())
            .subcommand(leaf_analyticsadmin_properties_conversion_events_patch())
    }

    // analyticsadmin.properties.create
    fn leaf_analyticsadmin_properties_create() -> Command {
        Command::new("analyticsadmin.properties.create")
            .visible_alias("create")
            .about("Creates a Google Analytics property with the specified location and attributes.")
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions.archive
    fn leaf_analyticsadmin_properties_custom_dimensions_archive() -> Command {
        Command::new("analyticsadmin.properties.customDimensions.archive")
            .visible_alias("archive")
            .about("Archives a CustomDimension on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the CustomDimension to archive. Example format: properties/1234/customDimensions/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions.create
    fn leaf_analyticsadmin_properties_custom_dimensions_create() -> Command {
        Command::new("analyticsadmin.properties.customDimensions.create")
            .visible_alias("create")
            .about("Creates a CustomDimension. Warning: It's not permissible to use this method to collect data on...")
            .long_about("Creates a CustomDimension. Warning: It's not permissible to use this method to collect data on individual users. In particular, sending user IDs in custom dimensions violates the Google Analytics Terms of Service.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions.get
    fn leaf_analyticsadmin_properties_custom_dimensions_get() -> Command {
        Command::new("analyticsadmin.properties.customDimensions.get")
            .visible_alias("get")
            .about("Lookup for a single CustomDimension.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the CustomDimension to get. Example format: properties/1234/customDimensions/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions.list
    fn leaf_analyticsadmin_properties_custom_dimensions_list() -> Command {
        Command::new("analyticsadmin.properties.customDimensions.list")
            .visible_alias("list")
            .about("Lists CustomDimensions on a property.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200 (higher values will be coer"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListCustomDimensions` call. Provide this to retrieve the subsequent page. When paginating, all other parameter"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions.patch
    fn leaf_analyticsadmin_properties_custom_dimensions_patch() -> Command {
        Command::new("analyticsadmin.properties.customDimensions.patch")
            .visible_alias("patch")
            .about("Updates a CustomDimension on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name for this CustomDimension resource. Format: properties/{property}/customDimensions/{customDimension}"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Omitted fields will not be updated. To replace the entire entity, use one path with the string \"*\" to match all fiel"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customDimensions
    fn group_analyticsadmin_properties_custom_dimensions() -> Command {
        Command::new("customDimensions")
            .about("Methods under analyticsadmin.properties.customDimensions")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_custom_dimensions_archive())
            .subcommand(leaf_analyticsadmin_properties_custom_dimensions_create())
            .subcommand(leaf_analyticsadmin_properties_custom_dimensions_get())
            .subcommand(leaf_analyticsadmin_properties_custom_dimensions_list())
            .subcommand(leaf_analyticsadmin_properties_custom_dimensions_patch())
    }

    // analyticsadmin.properties.customMetrics.archive
    fn leaf_analyticsadmin_properties_custom_metrics_archive() -> Command {
        Command::new("analyticsadmin.properties.customMetrics.archive")
            .visible_alias("archive")
            .about("Archives a CustomMetric on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the CustomMetric to archive. Example format: properties/1234/customMetrics/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customMetrics.create
    fn leaf_analyticsadmin_properties_custom_metrics_create() -> Command {
        Command::new("analyticsadmin.properties.customMetrics.create")
            .visible_alias("create")
            .about("Creates a CustomMetric.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customMetrics.get
    fn leaf_analyticsadmin_properties_custom_metrics_get() -> Command {
        Command::new("analyticsadmin.properties.customMetrics.get")
            .visible_alias("get")
            .about("Lookup for a single CustomMetric.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the CustomMetric to get. Example format: properties/1234/customMetrics/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customMetrics.list
    fn leaf_analyticsadmin_properties_custom_metrics_list() -> Command {
        Command::new("analyticsadmin.properties.customMetrics.list")
            .visible_alias("list")
            .about("Lists CustomMetrics on a property.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200 (higher values will be coerced to the"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("A page token, received from a previous `ListCustomMetrics` call. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customMetrics.patch
    fn leaf_analyticsadmin_properties_custom_metrics_patch() -> Command {
        Command::new("analyticsadmin.properties.customMetrics.patch")
            .visible_alias("patch")
            .about("Updates a CustomMetric on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name for this CustomMetric resource. Format: properties/{property}/customMetrics/{customMetric}"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Omitted fields will not be updated. To replace the entire entity, use one path with the string \"*\" to match all fiel"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.customMetrics
    fn group_analyticsadmin_properties_custom_metrics() -> Command {
        Command::new("customMetrics")
            .about("Methods under analyticsadmin.properties.customMetrics")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_custom_metrics_archive())
            .subcommand(leaf_analyticsadmin_properties_custom_metrics_create())
            .subcommand(leaf_analyticsadmin_properties_custom_metrics_get())
            .subcommand(leaf_analyticsadmin_properties_custom_metrics_list())
            .subcommand(leaf_analyticsadmin_properties_custom_metrics_patch())
    }

    // analyticsadmin.properties.dataStreams.create
    fn leaf_analyticsadmin_properties_data_streams_create() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.create")
            .visible_alias("create")
            .about("Creates a DataStream.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.delete
    fn leaf_analyticsadmin_properties_data_streams_delete() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.delete")
            .visible_alias("delete")
            .about("Deletes a DataStream on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the DataStream to delete. Example format: properties/1234/dataStreams/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.get
    fn leaf_analyticsadmin_properties_data_streams_get() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.get")
            .visible_alias("get")
            .about("Lookup for a single DataStream.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the DataStream to get. Example format: properties/1234/dataStreams/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.list
    fn leaf_analyticsadmin_properties_data_streams_list() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.list")
            .visible_alias("list")
            .about("Lists DataStreams on a property.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200 (higher values will be coerced to the"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("A page token, received from a previous `ListDataStreams` call. Provide this to retrieve the subsequent page. When paginating, all other parameters provided to `"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets.create
    fn leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_create() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.measurementProtocolSecrets.create")
            .visible_alias("create")
            .about("Creates a measurement protocol secret.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent resource where this secret will be created. Format: properties/{property}/dataStreams/{dataStream}"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets.delete
    fn leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_delete() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.measurementProtocolSecrets.delete")
            .visible_alias("delete")
            .about("Deletes target MeasurementProtocolSecret.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the MeasurementProtocolSecret to delete. Format: properties/{property}/dataStreams/{dataStream}/measurementProtocolSecrets/{measurementPro"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets.get
    fn leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_get() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.measurementProtocolSecrets.get")
            .visible_alias("get")
            .about("Lookup for a single MeasurementProtocolSecret.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the measurement protocol secret to lookup. Format: properties/{property}/dataStreams/{dataStream}/measurementProtocolSecrets/{measurementP"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets.list
    fn leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_list() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.measurementProtocolSecrets.list")
            .visible_alias("list")
            .about("Returns child MeasurementProtocolSecrets under the specified parent Property.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. If unspecified, at most 10 resources will be returned. The maximum value is 10. Higher values will be coerc"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListMeasurementProtocolSecrets` call. Provide this to retrieve the subsequent page. When paginating, all other"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the parent stream. Format: properties/{property}/dataStreams/{dataStream}/measurementProtocolSecrets"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets.patch
    fn leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_patch() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.measurementProtocolSecrets.patch")
            .visible_alias("patch")
            .about("Updates a measurement protocol secret.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of this secret. This secret may be a child of any type of stream. Format: properties/{property}/dataStreams/{dataStream}/measurementPr"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Omitted fields will not be updated."))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams.measurementProtocolSecrets
    fn group_analyticsadmin_properties_data_streams_measurement_protocol_secrets() -> Command {
        Command::new("measurementProtocolSecrets")
            .about("Methods under analyticsadmin.properties.dataStreams.measurementProtocolSecrets")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_create())
            .subcommand(leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_delete())
            .subcommand(leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_get())
            .subcommand(leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_list())
            .subcommand(leaf_analyticsadmin_properties_data_streams_measurement_protocol_secrets_patch())
    }

    // analyticsadmin.properties.dataStreams.patch
    fn leaf_analyticsadmin_properties_data_streams_patch() -> Command {
        Command::new("analyticsadmin.properties.dataStreams.patch")
            .visible_alias("patch")
            .about("Updates a DataStream on a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of this Data Stream. Format: properties/{property_id}/dataStreams/{stream_id} Example: \"properties/1000/dataStreams/2000\""))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Omitted fields will not be updated. To replace the entire entity, use one path with the string \"*\" to match all fiel"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.dataStreams
    fn group_analyticsadmin_properties_data_streams() -> Command {
        Command::new("dataStreams")
            .about("Methods under analyticsadmin.properties.dataStreams")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_data_streams_create())
            .subcommand(leaf_analyticsadmin_properties_data_streams_delete())
            .subcommand(leaf_analyticsadmin_properties_data_streams_get())
            .subcommand(leaf_analyticsadmin_properties_data_streams_list())
            .subcommand(group_analyticsadmin_properties_data_streams_measurement_protocol_secrets())
            .subcommand(leaf_analyticsadmin_properties_data_streams_patch())
    }

    // analyticsadmin.properties.delete
    fn leaf_analyticsadmin_properties_delete() -> Command {
        Command::new("analyticsadmin.properties.delete")
            .visible_alias("delete")
            .about("Marks target Property as soft-deleted (ie: \"trashed\") and returns it. This API does not have a...")
            .long_about("Marks target Property as soft-deleted (ie: \"trashed\") and returns it. This API does not have a method to restore soft-deleted properties. However, they can be restored using the Trash Can UI. If the properties are not restored before the expiration time, the Property and all child resources (eg: Goo")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the Property to soft-delete. Format: properties/{property_id} Example: \"properties/1000\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.firebaseLinks.create
    fn leaf_analyticsadmin_properties_firebase_links_create() -> Command {
        Command::new("analyticsadmin.properties.firebaseLinks.create")
            .visible_alias("create")
            .about("Creates a FirebaseLink. Properties can have at most one FirebaseLink.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Format: properties/{property_id} Example: `properties/1234`"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.firebaseLinks.delete
    fn leaf_analyticsadmin_properties_firebase_links_delete() -> Command {
        Command::new("analyticsadmin.properties.firebaseLinks.delete")
            .visible_alias("delete")
            .about("Deletes a FirebaseLink on a property")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Format: properties/{property_id}/firebaseLinks/{firebase_link_id} Example: `properties/1234/firebaseLinks/5678`"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.firebaseLinks.list
    fn leaf_analyticsadmin_properties_firebase_links_list() -> Command {
        Command::new("analyticsadmin.properties.firebaseLinks.list")
            .visible_alias("list")
            .about("Lists FirebaseLinks on a property. Properties can have at most one FirebaseLink.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. The service may return fewer than this value, even if there are additional pages. If unspecified, at most 5"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListFirebaseLinks` call. Provide this to retrieve the subsequent page. When paginating, all other parameters p"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Format: properties/{property_id} Example: `properties/1234`"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.firebaseLinks
    fn group_analyticsadmin_properties_firebase_links() -> Command {
        Command::new("firebaseLinks")
            .about("Methods under analyticsadmin.properties.firebaseLinks")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_firebase_links_create())
            .subcommand(leaf_analyticsadmin_properties_firebase_links_delete())
            .subcommand(leaf_analyticsadmin_properties_firebase_links_list())
    }

    // analyticsadmin.properties.get
    fn leaf_analyticsadmin_properties_get() -> Command {
        Command::new("analyticsadmin.properties.get")
            .visible_alias("get")
            .about("Lookup for a single GA Property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the property to lookup. Format: properties/{property_id} Example: \"properties/1000\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.getDataRetentionSettings
    fn leaf_analyticsadmin_properties_get_data_retention_settings() -> Command {
        Command::new("analyticsadmin.properties.getDataRetentionSettings")
            .visible_alias("getDataRetentionSettings")
            .about("Returns the singleton data retention settings for this property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the settings to lookup. Format: properties/{property}/dataRetentionSettings Example: \"properties/1000/dataRetentionSettings\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.googleAdsLinks.create
    fn leaf_analyticsadmin_properties_google_ads_links_create() -> Command {
        Command::new("analyticsadmin.properties.googleAdsLinks.create")
            .visible_alias("create")
            .about("Creates a GoogleAdsLink.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.googleAdsLinks.delete
    fn leaf_analyticsadmin_properties_google_ads_links_delete() -> Command {
        Command::new("analyticsadmin.properties.googleAdsLinks.delete")
            .visible_alias("delete")
            .about("Deletes a GoogleAdsLink on a property")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Example format: properties/1234/googleAdsLinks/5678"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.googleAdsLinks.list
    fn leaf_analyticsadmin_properties_google_ads_links_list() -> Command {
        Command::new("analyticsadmin.properties.googleAdsLinks.list")
            .visible_alias("list")
            .about("Lists GoogleAdsLinks on a property.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200 (higher values will be coer"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListGoogleAdsLinks` call. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Example format: properties/1234"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.googleAdsLinks.patch
    fn leaf_analyticsadmin_properties_google_ads_links_patch() -> Command {
        Command::new("analyticsadmin.properties.googleAdsLinks.patch")
            .visible_alias("patch")
            .about("Updates a GoogleAdsLink on a property")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Format: properties/{propertyId}/googleAdsLinks/{googleAdsLinkId} Note: googleAdsLinkId is not the Google Ads customer ID."))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (e.g., \"field_to_update\"). Omitted fields will not be updated. To replace the enti"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.googleAdsLinks
    fn group_analyticsadmin_properties_google_ads_links() -> Command {
        Command::new("googleAdsLinks")
            .about("Methods under analyticsadmin.properties.googleAdsLinks")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_google_ads_links_create())
            .subcommand(leaf_analyticsadmin_properties_google_ads_links_delete())
            .subcommand(leaf_analyticsadmin_properties_google_ads_links_list())
            .subcommand(leaf_analyticsadmin_properties_google_ads_links_patch())
    }

    // analyticsadmin.properties.keyEvents.create
    fn leaf_analyticsadmin_properties_key_events_create() -> Command {
        Command::new("analyticsadmin.properties.keyEvents.create")
            .visible_alias("create")
            .about("Creates a Key Event.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the parent property where this Key Event will be created. Format: properties/123"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.keyEvents.delete
    fn leaf_analyticsadmin_properties_key_events_delete() -> Command {
        Command::new("analyticsadmin.properties.keyEvents.delete")
            .visible_alias("delete")
            .about("Deletes a Key Event.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the Key Event to delete. Format: properties/{property}/keyEvents/{key_event} Example: \"properties/123/keyEvents/456\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.keyEvents.get
    fn leaf_analyticsadmin_properties_key_events_get() -> Command {
        Command::new("analyticsadmin.properties.keyEvents.get")
            .visible_alias("get")
            .about("Retrieves a single Key Event.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the Key Event to retrieve. Format: properties/{property}/keyEvents/{key_event} Example: \"properties/123/keyEvents/456\""))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.keyEvents.list
    fn leaf_analyticsadmin_properties_key_events_list() -> Command {
        Command::new("analyticsadmin.properties.keyEvents.list")
            .visible_alias("list")
            .about("Returns a list of Key Events in the specified parent property. Returns an empty list if no Key...")
            .long_about("Returns a list of Key Events in the specified parent property. Returns an empty list if no Key Events are found.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. If unspecified, at most 50 resources will be returned. The maximum value is 200; (higher values will be coe"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListKeyEvents` call. Provide this to retrieve the subsequent page. When paginating, all other parameters provi"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the parent property. Example: 'properties/123'"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.keyEvents.patch
    fn leaf_analyticsadmin_properties_key_events_patch() -> Command {
        Command::new("analyticsadmin.properties.keyEvents.patch")
            .visible_alias("patch")
            .about("Updates a Key Event.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Output only. Resource name of this key event. Format: properties/{property}/keyEvents/{key_event}"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (e.g., \"field_to_update\"). Omitted fields will not be updated. To replace the enti"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.keyEvents
    fn group_analyticsadmin_properties_key_events() -> Command {
        Command::new("keyEvents")
            .about("Methods under analyticsadmin.properties.keyEvents")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_key_events_create())
            .subcommand(leaf_analyticsadmin_properties_key_events_delete())
            .subcommand(leaf_analyticsadmin_properties_key_events_get())
            .subcommand(leaf_analyticsadmin_properties_key_events_list())
            .subcommand(leaf_analyticsadmin_properties_key_events_patch())
    }

    // analyticsadmin.properties.list
    fn leaf_analyticsadmin_properties_list() -> Command {
        Command::new("analyticsadmin.properties.list")
            .visible_alias("list")
            .about("Returns child Properties under the specified parent Account. Properties will be excluded if the...")
            .long_about("Returns child Properties under the specified parent Account. Properties will be excluded if the caller does not have access. Soft-deleted (ie: \"trashed\") properties are excluded by default. Returns an empty list if no relevant properties are found.")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Required. An expression for filtering the results of the request. Fields eligible for filtering are: `parent:`(The resource name of the parent account/property)"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. The service may return fewer than this value, even if there are additional pages. If unspecified, at most 5"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListProperties` call. Provide this to retrieve the subsequent page. When paginating, all other parameters prov"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include soft-deleted (ie: \"trashed\") Properties in the results. Properties can be inspected to determine whether they are deleted or not."))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.patch
    fn leaf_analyticsadmin_properties_patch() -> Command {
        Command::new("analyticsadmin.properties.patch")
            .visible_alias("patch")
            .about("Updates a property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of this property. Format: properties/{property_id} Example: \"properties/1000\""))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (e.g., \"field_to_update\"). Omitted fields will not be updated. To replace the enti"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.runAccessReport
    fn leaf_analyticsadmin_properties_run_access_report() -> Command {
        Command::new("analyticsadmin.properties.runAccessReport")
            .visible_alias("runAccessReport")
            .about("Returns a customized report of data access records. The report provides records of each time a user...")
            .long_about("Returns a customized report of data access records. The report provides records of each time a user reads Google Analytics reporting data. Access records are retained for up to 2 years. Data Access Reports can be requested for a property. Reports may be requested for any property, but dimensions tha")
            .arg(Arg::new("entity").long("entity").value_name("ENTITY").required(true)
                .help("The Data Access Report supports requesting at the property level or account level. If requested at the account level, Data Access Reports include all access for"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties.updateDataRetentionSettings
    fn leaf_analyticsadmin_properties_update_data_retention_settings() -> Command {
        Command::new("analyticsadmin.properties.updateDataRetentionSettings")
            .visible_alias("updateDataRetentionSettings")
            .about("Updates the singleton data retention settings for this property.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name for this DataRetentionSetting resource. Format: properties/{property}/dataRetentionSettings"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to be updated. Field names must be in snake case (e.g., \"field_to_update\"). Omitted fields will not be updated. To replace the enti"))
            .args(escape_hatch_args())
    }

    // analyticsadmin.properties
    fn group_analyticsadmin_properties() -> Command {
        Command::new("properties")
            .about("Methods under analyticsadmin.properties")
            .subcommand_required(true)
            .subcommand(leaf_analyticsadmin_properties_acknowledge_user_data_collection())
            .subcommand(group_analyticsadmin_properties_conversion_events())
            .subcommand(leaf_analyticsadmin_properties_create())
            .subcommand(group_analyticsadmin_properties_custom_dimensions())
            .subcommand(group_analyticsadmin_properties_custom_metrics())
            .subcommand(group_analyticsadmin_properties_data_streams())
            .subcommand(leaf_analyticsadmin_properties_delete())
            .subcommand(group_analyticsadmin_properties_firebase_links())
            .subcommand(leaf_analyticsadmin_properties_get())
            .subcommand(leaf_analyticsadmin_properties_get_data_retention_settings())
            .subcommand(group_analyticsadmin_properties_google_ads_links())
            .subcommand(group_analyticsadmin_properties_key_events())
            .subcommand(leaf_analyticsadmin_properties_list())
            .subcommand(leaf_analyticsadmin_properties_patch())
            .subcommand(leaf_analyticsadmin_properties_run_access_report())
            .subcommand(leaf_analyticsadmin_properties_update_data_retention_settings())
    }

    // analyticsadmin
    fn service_analyticsadmin() -> Command {
        Command::new("analyticsadmin")
            .about("Google Analytics Admin API operations (v1beta, 55 methods)")
            .subcommand_required(true)
            .subcommand(group_analyticsadmin_accounts())
            .subcommand(group_analyticsadmin_account_summaries())
            .subcommand(group_analyticsadmin_properties())
    }

    // analyticsdata.properties.audienceExports.create
    fn leaf_analyticsdata_properties_audience_exports_create() -> Command {
        Command::new("analyticsdata.properties.audienceExports.create")
            .visible_alias("create")
            .about("Creates an audience export for later retrieval. This method quickly returns the audience export's...")
            .long_about("Creates an audience export for later retrieval. This method quickly returns the audience export's resource name and initiates a long running asynchronous request to form an audience export. To export the users in an audience export, first create the audience export through this method and then send")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent resource where this audience export will be created. Format: `properties/{property}`"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.audienceExports.get
    fn leaf_analyticsdata_properties_audience_exports_get() -> Command {
        Command::new("analyticsdata.properties.audienceExports.get")
            .visible_alias("get")
            .about("Gets configuration metadata about a specific audience export. This method can be used to understand...")
            .long_about("Gets configuration metadata about a specific audience export. This method can be used to understand an audience export after it has been created. See Creating an Audience Export for an introduction to Audienc")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The audience export resource name. Format: `properties/{property}/audienceExports/{audience_export}`"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.audienceExports.list
    fn leaf_analyticsdata_properties_audience_exports_list() -> Command {
        Command::new("analyticsdata.properties.audienceExports.list")
            .visible_alias("list")
            .about("Lists all audience exports for a property. This method can be used for you to find and reuse...")
            .long_about("Lists all audience exports for a property. This method can be used for you to find and reuse existing audience exports rather than creating unnecessary new audience exports. The same audience can have multiple audience exports that represent the export of users that were in an audience on different")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of audience exports to return. The service may return fewer than this value. If unspecified, at most 200 audience exports will be r"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous `ListAudienceExports` call. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. All audience exports for this property will be listed in the response. Format: `properties/{property}`"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.audienceExports.query
    fn leaf_analyticsdata_properties_audience_exports_query() -> Command {
        Command::new("analyticsdata.properties.audienceExports.query")
            .visible_alias("query")
            .about("Retrieves an audience export of users. After creating an audience, the users are not immediately...")
            .long_about("Retrieves an audience export of users. After creating an audience, the users are not immediately available for exporting. First, a request to `CreateAudienceExport` is necessary to create an audience export of users, and then second, this method is used to retrieve the users in the audience export.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the audience export to retrieve users from. Format: `properties/{property}/audienceExports/{audience_export}`"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.audienceExports
    fn group_analyticsdata_properties_audience_exports() -> Command {
        Command::new("audienceExports")
            .about("Methods under analyticsdata.properties.audienceExports")
            .subcommand_required(true)
            .subcommand(leaf_analyticsdata_properties_audience_exports_create())
            .subcommand(leaf_analyticsdata_properties_audience_exports_get())
            .subcommand(leaf_analyticsdata_properties_audience_exports_list())
            .subcommand(leaf_analyticsdata_properties_audience_exports_query())
    }

    // analyticsdata.properties.batchRunPivotReports
    fn leaf_analyticsdata_properties_batch_run_pivot_reports() -> Command {
        Command::new("analyticsdata.properties.batchRunPivotReports")
            .visible_alias("batchRunPivotReports")
            .about("Returns multiple pivot reports in a batch. All reports must be for the same Google Analytics...")
            .long_about("Returns multiple pivot reports in a batch. All reports must be for the same Google Analytics property.")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. Specified in the URL path and not the body. To learn more, see [where to find your Property ID]"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.batchRunReports
    fn leaf_analyticsdata_properties_batch_run_reports() -> Command {
        Command::new("analyticsdata.properties.batchRunReports")
            .visible_alias("batchRunReports")
            .about("Returns multiple reports in a batch. All reports must be for the same Google Analytics property.")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. Specified in the URL path and not the body. To learn more, see [where to find your Property ID]"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.checkCompatibility
    fn leaf_analyticsdata_properties_check_compatibility() -> Command {
        Command::new("analyticsdata.properties.checkCompatibility")
            .visible_alias("checkCompatibility")
            .about("This compatibility method lists dimensions and metrics that can be added to a report request and...")
            .long_about("This compatibility method lists dimensions and metrics that can be added to a report request and maintain compatibility. This method fails if the request's dimensions and metrics are incompatible. In Google Analytics, reports fail if they request incompatible dimensions and/or metrics; in that case,")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. To learn more, see [where to find your Property ID](https://developers.google.com/analytics/dev"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.getMetadata
    fn leaf_analyticsdata_properties_get_metadata() -> Command {
        Command::new("analyticsdata.properties.getMetadata")
            .visible_alias("getMetadata")
            .about("Returns metadata for dimensions and metrics available in reporting methods. Used to explore the...")
            .long_about("Returns metadata for dimensions and metrics available in reporting methods. Used to explore the dimensions and metrics. In this method, a Google Analytics property identifier is specified in the request, and the metadata response includes Custom dimensions and metrics as well as Universal metadata.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the metadata to retrieve. This name field is specified in the URL path and not URL parameters. Property is a numeric Google Analy"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.runPivotReport
    fn leaf_analyticsdata_properties_run_pivot_report() -> Command {
        Command::new("analyticsdata.properties.runPivotReport")
            .visible_alias("runPivotReport")
            .about("Returns a customized pivot report of your Google Analytics event data. Pivot reports are more...")
            .long_about("Returns a customized pivot report of your Google Analytics event data. Pivot reports are more advanced and expressive formats than regular reports. In a pivot report, dimensions are only visible if they are included in a pivot. Multiple pivots can be specified to further dissect your data.")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. Specified in the URL path and not the body. To learn more, see [where to find your Property ID]"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.runRealtimeReport
    fn leaf_analyticsdata_properties_run_realtime_report() -> Command {
        Command::new("analyticsdata.properties.runRealtimeReport")
            .visible_alias("runRealtimeReport")
            .about("Returns a customized report of realtime event data for your property. Events appear in realtime...")
            .long_about("Returns a customized report of realtime event data for your property. Events appear in realtime reports seconds after they have been sent to the Google Analytics. Realtime reports show events and usage data for the periods of time ranging from the present moment to 30 minutes ago (up to 60 minutes f")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. Specified in the URL path and not the body. To learn more, see [where to find your Property ID]"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties.runReport
    fn leaf_analyticsdata_properties_run_report() -> Command {
        Command::new("analyticsdata.properties.runReport")
            .visible_alias("runReport")
            .about("Returns a customized report of your Google Analytics event data. Reports contain statistics derived...")
            .long_about("Returns a customized report of your Google Analytics event data. Reports contain statistics derived from data collected by the Google Analytics tracking code. The data returned from the API is as a table with columns for the requested dimensions and metrics. Metrics are individual measurements of us")
            .arg(Arg::new("property").long("property").value_name("PROPERTY").required(true)
                .help("A Google Analytics property identifier whose events are tracked. Specified in the URL path and not the body. To learn more, see [where to find your Property ID]"))
            .args(escape_hatch_args())
    }

    // analyticsdata.properties
    fn group_analyticsdata_properties() -> Command {
        Command::new("properties")
            .about("Methods under analyticsdata.properties")
            .subcommand_required(true)
            .subcommand(group_analyticsdata_properties_audience_exports())
            .subcommand(leaf_analyticsdata_properties_batch_run_pivot_reports())
            .subcommand(leaf_analyticsdata_properties_batch_run_reports())
            .subcommand(leaf_analyticsdata_properties_check_compatibility())
            .subcommand(leaf_analyticsdata_properties_get_metadata())
            .subcommand(leaf_analyticsdata_properties_run_pivot_report())
            .subcommand(leaf_analyticsdata_properties_run_realtime_report())
            .subcommand(leaf_analyticsdata_properties_run_report())
    }

    // analyticsdata
    fn service_analyticsdata() -> Command {
        Command::new("analyticsdata")
            .about("Google Analytics Data API operations (v1beta, 11 methods)")
            .subcommand_required(true)
            .subcommand(group_analyticsdata_properties())
    }

    // calendar.acl.delete
    fn leaf_calendar_acl_delete() -> Command {
        Command::new("calendar.acl.delete")
            .visible_alias("delete")
            .about("Deletes an access control rule.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("rule-id").long("rule-id").value_name("RULE_ID").required(true)
                .help("ACL rule identifier."))
            .args(escape_hatch_args())
    }

    // calendar.acl.get
    fn leaf_calendar_acl_get() -> Command {
        Command::new("calendar.acl.get")
            .visible_alias("get")
            .about("Returns an access control rule.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("rule-id").long("rule-id").value_name("RULE_ID").required(true)
                .help("ACL rule identifier."))
            .args(escape_hatch_args())
    }

    // calendar.acl.insert
    fn leaf_calendar_acl_insert() -> Command {
        Command::new("calendar.acl.insert")
            .visible_alias("insert")
            .about("Creates an access control rule.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Whether to send notifications about the calendar sharing change. Optional. The default is True."))
            .args(escape_hatch_args())
    }

    // calendar.acl.list
    fn leaf_calendar_acl_list() -> Command {
        Command::new("calendar.acl.list")
            .visible_alias("list")
            .about("Returns the rules in the access control list for the calendar.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted ACLs in the result. Deleted ACLs are represented by role equal to \"none\". Deleted ACLs will always be included if syncToken is provid"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.acl.patch
    fn leaf_calendar_acl_patch() -> Command {
        Command::new("calendar.acl.patch")
            .visible_alias("patch")
            .about("Updates an access control rule. This method supports patch semantics.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("rule-id").long("rule-id").value_name("RULE_ID").required(true)
                .help("ACL rule identifier."))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Whether to send notifications about the calendar sharing change. Note that there are no notifications on access removal. Optional. The default is True."))
            .args(escape_hatch_args())
    }

    // calendar.acl.update
    fn leaf_calendar_acl_update() -> Command {
        Command::new("calendar.acl.update")
            .visible_alias("update")
            .about("Updates an access control rule.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("rule-id").long("rule-id").value_name("RULE_ID").required(true)
                .help("ACL rule identifier."))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Whether to send notifications about the calendar sharing change. Note that there are no notifications on access removal. Optional. The default is True."))
            .args(escape_hatch_args())
    }

    // calendar.acl.watch
    fn leaf_calendar_acl_watch() -> Command {
        Command::new("calendar.acl.watch")
            .visible_alias("watch")
            .about("Watch for changes to ACL resources.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted ACLs in the result. Deleted ACLs are represented by role equal to \"none\". Deleted ACLs will always be included if syncToken is provid"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.acl
    fn group_calendar_acl() -> Command {
        Command::new("acl")
            .about("Methods under calendar.acl")
            .subcommand_required(true)
            .subcommand(leaf_calendar_acl_delete())
            .subcommand(leaf_calendar_acl_get())
            .subcommand(leaf_calendar_acl_insert())
            .subcommand(leaf_calendar_acl_list())
            .subcommand(leaf_calendar_acl_patch())
            .subcommand(leaf_calendar_acl_update())
            .subcommand(leaf_calendar_acl_watch())
    }

    // calendar.calendarList.delete
    fn leaf_calendar_calendar_list_delete() -> Command {
        Command::new("calendar.calendarList.delete")
            .visible_alias("delete")
            .about("Removes a calendar from the user's calendar list.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.get
    fn leaf_calendar_calendar_list_get() -> Command {
        Command::new("calendar.calendarList.get")
            .visible_alias("get")
            .about("Returns a calendar from the user's calendar list.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.insert
    fn leaf_calendar_calendar_list_insert() -> Command {
        Command::new("calendar.calendarList.insert")
            .visible_alias("insert")
            .about("Inserts an existing calendar into the user's calendar list.")
            .arg(Arg::new("color-rgb-format").long("color-rgb-format").action(ArgAction::SetTrue)
                .help("Whether to use the foregroundColor and backgroundColor fields to write the calendar colors (RGB). If this feature is used, the index-based colorId field will be"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.list
    fn leaf_calendar_calendar_list_list() -> Command {
        Command::new("calendar.calendarList.list")
            .visible_alias("list")
            .about("Returns the calendars on the user's calendar list.")
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("min-access-role").long("min-access-role").value_name("MIN_ACCESS_ROLE").value_parser(["freeBusyReader", "owner", "reader", "writer", "writerWithoutPrivateAccess"])
                .help("The minimum access role for the user in the returned entries. Optional. The default is no restriction."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted calendar list entries in the result. Optional. The default is False."))
            .arg(Arg::new("show-hidden").long("show-hidden").action(ArgAction::SetTrue)
                .help("Whether to show hidden entries. Optional. The default is False."))
            .arg(Arg::new("show-own-organization-only").long("show-own-organization-only").action(ArgAction::SetTrue)
                .help("Whether to show only entries for calendars from the organization. This parameter is only applicable to Google Workspace users. Optional. The default is False."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.patch
    fn leaf_calendar_calendar_list_patch() -> Command {
        Command::new("calendar.calendarList.patch")
            .visible_alias("patch")
            .about("Updates an existing calendar on the user's calendar list. This method supports patch semantics.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("color-rgb-format").long("color-rgb-format").action(ArgAction::SetTrue)
                .help("Whether to use the foregroundColor and backgroundColor fields to write the calendar colors (RGB). If this feature is used, the index-based colorId field will be"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.update
    fn leaf_calendar_calendar_list_update() -> Command {
        Command::new("calendar.calendarList.update")
            .visible_alias("update")
            .about("Updates an existing calendar on the user's calendar list.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("color-rgb-format").long("color-rgb-format").action(ArgAction::SetTrue)
                .help("Whether to use the foregroundColor and backgroundColor fields to write the calendar colors (RGB). If this feature is used, the index-based colorId field will be"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList.watch
    fn leaf_calendar_calendar_list_watch() -> Command {
        Command::new("calendar.calendarList.watch")
            .visible_alias("watch")
            .about("Watch for changes to CalendarList resources.")
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("min-access-role").long("min-access-role").value_name("MIN_ACCESS_ROLE").value_parser(["freeBusyReader", "owner", "reader", "writer", "writerWithoutPrivateAccess"])
                .help("The minimum access role for the user in the returned entries. Optional. The default is no restriction."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted calendar list entries in the result. Optional. The default is False."))
            .arg(Arg::new("show-hidden").long("show-hidden").action(ArgAction::SetTrue)
                .help("Whether to show hidden entries. Optional. The default is False."))
            .arg(Arg::new("show-own-organization-only").long("show-own-organization-only").action(ArgAction::SetTrue)
                .help("Whether to show only entries for calendars from the organization. This parameter is only applicable to Google Workspace users. Optional. The default is False."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.calendarList
    fn group_calendar_calendar_list() -> Command {
        Command::new("calendarList")
            .about("Methods under calendar.calendarList")
            .subcommand_required(true)
            .subcommand(leaf_calendar_calendar_list_delete())
            .subcommand(leaf_calendar_calendar_list_get())
            .subcommand(leaf_calendar_calendar_list_insert())
            .subcommand(leaf_calendar_calendar_list_list())
            .subcommand(leaf_calendar_calendar_list_patch())
            .subcommand(leaf_calendar_calendar_list_update())
            .subcommand(leaf_calendar_calendar_list_watch())
    }

    // calendar.calendars.clear
    fn leaf_calendar_calendars_clear() -> Command {
        Command::new("calendar.calendars.clear")
            .visible_alias("clear")
            .about("Clears a primary calendar. This operation deletes all events associated with the primary calendar...")
            .long_about("Clears a primary calendar. This operation deletes all events associated with the primary calendar of an account.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendars.delete
    fn leaf_calendar_calendars_delete() -> Command {
        Command::new("calendar.calendars.delete")
            .visible_alias("delete")
            .about("Deletes a secondary calendar. Use calendars.clear for clearing all events on primary calendars.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendars.get
    fn leaf_calendar_calendars_get() -> Command {
        Command::new("calendar.calendars.get")
            .visible_alias("get")
            .about("Returns metadata for a calendar.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendars.insert
    fn leaf_calendar_calendars_insert() -> Command {
        Command::new("calendar.calendars.insert")
            .visible_alias("insert")
            .about("Creates a secondary calendar. The authenticated user for the request is made the data owner of the...")
            .long_about("Creates a secondary calendar. The authenticated user for the request is made the data owner of the new calendar. Note: We recommend to authenticate as the intended data owner of the calendar. You can use domain-wide delegation of authority to allow applications to act on behalf of a specific user. D")
            .args(escape_hatch_args())
    }

    // calendar.calendars.patch
    fn leaf_calendar_calendars_patch() -> Command {
        Command::new("calendar.calendars.patch")
            .visible_alias("patch")
            .about("Updates metadata for a calendar. This method supports patch semantics.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendars.transferOwnership
    fn leaf_calendar_calendars_transfer_ownership() -> Command {
        Command::new("calendar.calendars.transferOwnership")
            .visible_alias("transferOwnership")
            .about("Transfers a secondary calendar between users within a Google Workspace organization. Requires user...")
            .long_about("Transfers a secondary calendar between users within a Google Workspace organization. Requires user authentication with Manage Calendars administrator privilege, and one of the following authorization scopes: - https://www.googleapis.com/auth/calendar - https://www.googleapis.com/auth/calendar.calend")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs, call the calendarList.list method."))
            .arg(Arg::new("new-data-owner").long("new-data-owner").value_name("NEW_DATA_OWNER").required(true)
                .help("The email address of a user who will become the data owner of the calendar."))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue).required(true)
                .help("When true, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator with the Manage"))
            .args(escape_hatch_args())
    }

    // calendar.calendars.update
    fn leaf_calendar_calendars_update() -> Command {
        Command::new("calendar.calendars.update")
            .visible_alias("update")
            .about("Updates metadata for a calendar.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .args(escape_hatch_args())
    }

    // calendar.calendars
    fn group_calendar_calendars() -> Command {
        Command::new("calendars")
            .about("Methods under calendar.calendars")
            .subcommand_required(true)
            .subcommand(leaf_calendar_calendars_clear())
            .subcommand(leaf_calendar_calendars_delete())
            .subcommand(leaf_calendar_calendars_get())
            .subcommand(leaf_calendar_calendars_insert())
            .subcommand(leaf_calendar_calendars_patch())
            .subcommand(leaf_calendar_calendars_transfer_ownership())
            .subcommand(leaf_calendar_calendars_update())
    }

    // calendar.channels.stop
    fn leaf_calendar_channels_stop() -> Command {
        Command::new("calendar.channels.stop")
            .visible_alias("stop")
            .about("Stop watching resources through this channel")
            .args(escape_hatch_args())
    }

    // calendar.channels
    fn group_calendar_channels() -> Command {
        Command::new("channels")
            .about("Methods under calendar.channels")
            .subcommand_required(true)
            .subcommand(leaf_calendar_channels_stop())
    }

    // calendar.colors.get
    fn leaf_calendar_colors_get() -> Command {
        Command::new("calendar.colors.get")
            .visible_alias("get")
            .about("Returns the color definitions for calendars and events.")
            .args(escape_hatch_args())
    }

    // calendar.colors
    fn group_calendar_colors() -> Command {
        Command::new("colors")
            .about("Methods under calendar.colors")
            .subcommand_required(true)
            .subcommand(leaf_calendar_colors_get())
    }

    // calendar.events.delete
    fn leaf_calendar_events_delete() -> Command {
        Command::new("calendar.events.delete")
            .visible_alias("delete")
            .about("Deletes an event.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Event identifier."))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the deletion of the event. Note that some emails might still be sent even if you"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Guests who should receive notifications about the deletion of the event."))
            .args(escape_hatch_args())
    }

    // calendar.events.get
    fn leaf_calendar_events_get() -> Command {
        Command::new("calendar.events.get")
            .visible_alias("get")
            .about("Returns an event based on its Google Calendar ID. To retrieve an event using its iCalendar ID, call...")
            .long_about("Returns an event based on its Google Calendar ID. To retrieve an event using its iCalendar ID, call the events.list method using the iCalUID parameter.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored. A value will always be returned in the email field for the organizer, creator and attendees, even if no real email address is available"))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Event identifier."))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("time-zone").long("time-zone").value_name("TIME_ZONE")
                .help("Time zone used in the response. Optional. The default is the time zone of the calendar."))
            .args(escape_hatch_args())
    }

    // calendar.events.import
    fn leaf_calendar_events_import() -> Command {
        Command::new("calendar.events.import")
            .visible_alias("import")
            .about("Imports an event. This operation is used to add a private copy of an existing event to a calendar...")
            .long_about("Imports an event. This operation is used to add a private copy of an existing event to a calendar. Only events with an eventType of default may be imported. Deprecated behavior: If a non-default event is imported, its type will be changed to default and any event-type-specific properties it may have")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("conference-data-version").long("conference-data-version").value_name("CONFERENCE_DATA_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of conference data supported by the API client. Version 0 assumes no conference data support and ignores conference data in the event's body. Ver"))
            .arg(Arg::new("event-label-version").long("event-label-version").value_name("EVENT_LABEL_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of the event label feature supported by the API client. Version 0 assumes no event label support and processes the colorId field for color manage"))
            .arg(Arg::new("supports-attachments").long("supports-attachments").action(ArgAction::SetTrue)
                .help("Whether API client performing operation supports event attachments. Optional. The default is False."))
            .args(escape_hatch_args())
    }

    // calendar.events.insert
    fn leaf_calendar_events_insert() -> Command {
        Command::new("calendar.events.insert")
            .visible_alias("insert")
            .about("Creates an event.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("conference-data-version").long("conference-data-version").value_name("CONFERENCE_DATA_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of conference data supported by the API client. Version 0 assumes no conference data support and ignores conference data in the event's body. Ver"))
            .arg(Arg::new("event-label-version").long("event-label-version").value_name("EVENT_LABEL_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of the event label feature supported by the API client. Version 0 assumes no event label support and processes the colorId field for color manage"))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the creation of the new event. Note that some emails might still be sent even if"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Whether to send notifications about the creation of the new event. Note that some emails might still be sent. The default is false."))
            .arg(Arg::new("supports-attachments").long("supports-attachments").action(ArgAction::SetTrue)
                .help("Whether API client performing operation supports event attachments. Optional. The default is False."))
            .args(escape_hatch_args())
    }

    // calendar.events.instances
    fn leaf_calendar_events_instances() -> Command {
        Command::new("calendar.events.instances")
            .visible_alias("instances")
            .about("Returns instances of the specified recurring event.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored. A value will always be returned in the email field for the organizer, creator and attendees, even if no real email address is available"))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Recurring event identifier."))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of events returned on one result page. By default the value is 250 events. The page size can never be larger than 2500 events. Optional."))
            .arg(Arg::new("original-start").long("original-start").value_name("ORIGINAL_START")
                .help("The original start time of the instance in the result. Optional."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted events (with status equals \"cancelled\") in the result. Cancelled instances of recurring events will still be included if singleEvents"))
            .arg(Arg::new("time-max").long("time-max").value_name("TIME_MAX")
                .help("Upper bound (exclusive) for an event's start time to filter by. Optional. The default is not to filter by start time. Must be an RFC3339 timestamp with mandator"))
            .arg(Arg::new("time-min").long("time-min").value_name("TIME_MIN")
                .help("Lower bound (inclusive) for an event's end time to filter by. Optional. The default is not to filter by end time. Must be an RFC3339 timestamp with mandatory ti"))
            .arg(Arg::new("time-zone").long("time-zone").value_name("TIME_ZONE")
                .help("Time zone used in the response. Optional. The default is the time zone of the calendar."))
            .args(escape_hatch_args())
    }

    // calendar.events.list
    fn leaf_calendar_events_list() -> Command {
        Command::new("calendar.events.list")
            .visible_alias("list")
            .about("Returns events on the specified calendar.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored."))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("event-types").long("event-types").value_name("EVENT_TYPES").action(ArgAction::Append)
                .help("Event types to return. Optional. This parameter can be repeated multiple times to return events of different types. If unset, returns all event types."))
            .arg(Arg::new("i-cal-uid").long("i-cal-uid").value_name("I_CAL_UID")
                .help("Specifies an event ID in the iCalendar format to be provided in the response. Optional. Use this if you want to search for an event by its iCalendar ID."))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of events returned on one result page. The number of events in the resulting page may be less than this value, or none at all, even if there are"))
            .arg(Arg::new("order-by").long("order-by").value_name("ORDER_BY").value_parser(["startTime", "updated"])
                .help("The order of the events returned in the result. Optional. The default is an unspecified, stable order."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("private-extended-property").long("private-extended-property").value_name("PRIVATE_EXTENDED_PROPERTY").action(ArgAction::Append)
                .help("Extended properties constraint specified as propertyName=value. Matches only private properties. This parameter might be repeated multiple times to return event"))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Free text search terms to find events that match these terms in the following fields: - summary - description - location - attendee's displayName - attendee's e"))
            .arg(Arg::new("shared-extended-property").long("shared-extended-property").value_name("SHARED_EXTENDED_PROPERTY").action(ArgAction::Append)
                .help("Extended properties constraint specified as propertyName=value. Matches only shared properties. This parameter might be repeated multiple times to return events"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted events (with status equals \"cancelled\") in the result. Cancelled instances of recurring events (but not the underlying recurring even"))
            .arg(Arg::new("show-hidden-invitations").long("show-hidden-invitations").action(ArgAction::SetTrue)
                .help("Whether to include hidden invitations in the result. Optional. The default is False."))
            .arg(Arg::new("single-events").long("single-events").action(ArgAction::SetTrue)
                .help("Whether to expand recurring events into instances and only return single one-off events and instances of recurring events, but not the underlying recurring even"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .arg(Arg::new("time-max").long("time-max").value_name("TIME_MAX")
                .help("Upper bound (exclusive) for an event's start time to filter by. Optional. The default is not to filter by start time. Must be an RFC3339 timestamp with mandator"))
            .arg(Arg::new("time-min").long("time-min").value_name("TIME_MIN")
                .help("Lower bound (exclusive) for an event's end time to filter by. Optional. The default is not to filter by end time. Must be an RFC3339 timestamp with mandatory ti"))
            .arg(Arg::new("time-zone").long("time-zone").value_name("TIME_ZONE")
                .help("Time zone used in the response. Optional. The default is the time zone of the calendar."))
            .arg(Arg::new("updated-min").long("updated-min").value_name("UPDATED_MIN")
                .help("Lower bound for an event's last modification time (as a RFC3339 timestamp) to filter by. When specified, entries deleted since this time will always be included"))
            .args(escape_hatch_args())
    }

    // calendar.events.move
    fn leaf_calendar_events_move() -> Command {
        Command::new("calendar.events.move")
            .visible_alias("move")
            .about("Moves an event to another calendar, i.e. changes an event's organizer. Note that only default...")
            .long_about("Moves an event to another calendar, i.e. changes an event's organizer. Note that only default events can be moved; birthday, focusTime, fromGmail, outOfOffice and workingLocation events cannot be moved.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier of the source calendar where the event currently is on."))
            .arg(Arg::new("destination").long("destination").value_name("DESTINATION").required(true)
                .help("Calendar identifier of the target calendar where the event is to be moved to."))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Event identifier."))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the change of the event's organizer. Note that some emails might still be sent e"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Guests who should receive notifications about the change of the event's organizer."))
            .args(escape_hatch_args())
    }

    // calendar.events.patch
    fn leaf_calendar_events_patch() -> Command {
        Command::new("calendar.events.patch")
            .visible_alias("patch")
            .about("Updates an event. This method supports patch semantics.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored. A value will always be returned in the email field for the organizer, creator and attendees, even if no real email address is available"))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("conference-data-version").long("conference-data-version").value_name("CONFERENCE_DATA_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of conference data supported by the API client. Version 0 assumes no conference data support and ignores conference data in the event's body. Ver"))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Event identifier."))
            .arg(Arg::new("event-label-version").long("event-label-version").value_name("EVENT_LABEL_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of the event label feature supported by the API client. Version 0 assumes no event label support and processes the colorId field for color manage"))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the event update (for example, description changes, etc.). Note that some emails"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Guests who should receive notifications about the event update (for example, title changes, etc.)."))
            .arg(Arg::new("supports-attachments").long("supports-attachments").action(ArgAction::SetTrue)
                .help("Whether API client performing operation supports event attachments. Optional. The default is False."))
            .args(escape_hatch_args())
    }

    // calendar.events.quickAdd
    fn leaf_calendar_events_quick_add() -> Command {
        Command::new("calendar.events.quickAdd")
            .visible_alias("quickAdd")
            .about("Creates an event based on a simple text string.")
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the creation of the event. Note that some emails might still be sent even if you"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Guests who should receive notifications about the creation of the new event."))
            .arg(Arg::new("text").long("text").value_name("TEXT").required(true)
                .help("The text describing the event to be created."))
            .args(escape_hatch_args())
    }

    // calendar.events.update
    fn leaf_calendar_events_update() -> Command {
        Command::new("calendar.events.update")
            .visible_alias("update")
            .about("Updates an event.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored. A value will always be returned in the email field for the organizer, creator and attendees, even if no real email address is available"))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("conference-data-version").long("conference-data-version").value_name("CONFERENCE_DATA_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of conference data supported by the API client. Version 0 assumes no conference data support and ignores conference data in the event's body. Ver"))
            .arg(Arg::new("event-id").long("event-id").value_name("EVENT_ID").required(true)
                .help("Event identifier."))
            .arg(Arg::new("event-label-version").long("event-label-version").value_name("EVENT_LABEL_VERSION").value_parser(clap::value_parser!(i64))
                .help("Version number of the event label feature supported by the API client. Version 0 assumes no event label support and processes the colorId field for color manage"))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("send-notifications").long("send-notifications").action(ArgAction::SetTrue)
                .help("Deprecated. Please use sendUpdates instead. Whether to send notifications about the event update (for example, description changes, etc.). Note that some emails"))
            .arg(Arg::new("send-updates").long("send-updates").value_name("SEND_UPDATES").value_parser(["all", "externalOnly", "none"])
                .help("Guests who should receive notifications about the event update (for example, title changes, etc.)."))
            .arg(Arg::new("supports-attachments").long("supports-attachments").action(ArgAction::SetTrue)
                .help("Whether API client performing operation supports event attachments. Optional. The default is False."))
            .args(escape_hatch_args())
    }

    // calendar.events.watch
    fn leaf_calendar_events_watch() -> Command {
        Command::new("calendar.events.watch")
            .visible_alias("watch")
            .about("Watch for changes to Events resources.")
            .arg(Arg::new("always-include-email").long("always-include-email").action(ArgAction::SetTrue)
                .help("Deprecated and ignored."))
            .arg(Arg::new("calendar-id").long("calendar-id").value_name("CALENDAR_ID").required(true)
                .help("Calendar identifier. To retrieve calendar IDs call the calendarList.list method. If you want to access the primary calendar of the currently logged in user, use"))
            .arg(Arg::new("event-types").long("event-types").value_name("EVENT_TYPES").action(ArgAction::Append)
                .help("Event types to return. Optional. This parameter can be repeated multiple times to return events of different types. If unset, returns all event types."))
            .arg(Arg::new("i-cal-uid").long("i-cal-uid").value_name("I_CAL_UID")
                .help("Specifies an event ID in the iCalendar format to be provided in the response. Optional. Use this if you want to search for an event by its iCalendar ID."))
            .arg(Arg::new("max-attendees").long("max-attendees").value_name("MAX_ATTENDEES").value_parser(clap::value_parser!(i64))
                .help("The maximum number of attendees to include in the response. If there are more than the specified number of attendees, only the participant is returned. Optional"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of events returned on one result page. The number of events in the resulting page may be less than this value, or none at all, even if there are"))
            .arg(Arg::new("order-by").long("order-by").value_name("ORDER_BY").value_parser(["startTime", "updated"])
                .help("The order of the events returned in the result. Optional. The default is an unspecified, stable order."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("private-extended-property").long("private-extended-property").value_name("PRIVATE_EXTENDED_PROPERTY").action(ArgAction::Append)
                .help("Extended properties constraint specified as propertyName=value. Matches only private properties. This parameter might be repeated multiple times to return event"))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Free text search terms to find events that match these terms in the following fields: - summary - description - location - attendee's displayName - attendee's e"))
            .arg(Arg::new("shared-extended-property").long("shared-extended-property").value_name("SHARED_EXTENDED_PROPERTY").action(ArgAction::Append)
                .help("Extended properties constraint specified as propertyName=value. Matches only shared properties. This parameter might be repeated multiple times to return events"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted events (with status equals \"cancelled\") in the result. Cancelled instances of recurring events (but not the underlying recurring even"))
            .arg(Arg::new("show-hidden-invitations").long("show-hidden-invitations").action(ArgAction::SetTrue)
                .help("Whether to include hidden invitations in the result. Optional. The default is False."))
            .arg(Arg::new("single-events").long("single-events").action(ArgAction::SetTrue)
                .help("Whether to expand recurring events into instances and only return single one-off events and instances of recurring events, but not the underlying recurring even"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .arg(Arg::new("time-max").long("time-max").value_name("TIME_MAX")
                .help("Upper bound (exclusive) for an event's start time to filter by. Optional. The default is not to filter by start time. Must be an RFC3339 timestamp with mandator"))
            .arg(Arg::new("time-min").long("time-min").value_name("TIME_MIN")
                .help("Lower bound (exclusive) for an event's end time to filter by. Optional. The default is not to filter by end time. Must be an RFC3339 timestamp with mandatory ti"))
            .arg(Arg::new("time-zone").long("time-zone").value_name("TIME_ZONE")
                .help("Time zone used in the response. Optional. The default is the time zone of the calendar."))
            .arg(Arg::new("updated-min").long("updated-min").value_name("UPDATED_MIN")
                .help("Lower bound for an event's last modification time (as a RFC3339 timestamp) to filter by. When specified, entries deleted since this time will always be included"))
            .args(escape_hatch_args())
    }

    // calendar.events
    fn group_calendar_events() -> Command {
        Command::new("events")
            .about("Methods under calendar.events")
            .subcommand_required(true)
            .subcommand(leaf_calendar_events_delete())
            .subcommand(leaf_calendar_events_get())
            .subcommand(leaf_calendar_events_import())
            .subcommand(leaf_calendar_events_insert())
            .subcommand(leaf_calendar_events_instances())
            .subcommand(leaf_calendar_events_list())
            .subcommand(leaf_calendar_events_move())
            .subcommand(leaf_calendar_events_patch())
            .subcommand(leaf_calendar_events_quick_add())
            .subcommand(leaf_calendar_events_update())
            .subcommand(leaf_calendar_events_watch())
    }

    // calendar.freebusy.query
    fn leaf_calendar_freebusy_query() -> Command {
        Command::new("calendar.freebusy.query")
            .visible_alias("query")
            .about("Returns free/busy information for a set of calendars.")
            .args(escape_hatch_args())
    }

    // calendar.freebusy
    fn group_calendar_freebusy() -> Command {
        Command::new("freebusy")
            .about("Methods under calendar.freebusy")
            .subcommand_required(true)
            .subcommand(leaf_calendar_freebusy_query())
    }

    // calendar.settings.get
    fn leaf_calendar_settings_get() -> Command {
        Command::new("calendar.settings.get")
            .visible_alias("get")
            .about("Returns a single user setting.")
            .arg(Arg::new("setting").long("setting").value_name("SETTING").required(true)
                .help("The id of the user setting."))
            .args(escape_hatch_args())
    }

    // calendar.settings.list
    fn leaf_calendar_settings_list() -> Command {
        Command::new("calendar.settings.list")
            .visible_alias("list")
            .about("Returns all user settings for the authenticated user.")
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.settings.watch
    fn leaf_calendar_settings_watch() -> Command {
        Command::new("calendar.settings.watch")
            .visible_alias("watch")
            .about("Watch for changes to Settings resources.")
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of entries returned on one result page. By default the value is 100 entries. The page size can never be larger than 250 entries. Optional."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying which result page to return. Optional."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Token obtained from the nextSyncToken field returned on the last page of results from the previous list request. It makes the result of this list request contai"))
            .args(escape_hatch_args())
    }

    // calendar.settings
    fn group_calendar_settings() -> Command {
        Command::new("settings")
            .about("Methods under calendar.settings")
            .subcommand_required(true)
            .subcommand(leaf_calendar_settings_get())
            .subcommand(leaf_calendar_settings_list())
            .subcommand(leaf_calendar_settings_watch())
    }

    // calendar
    fn service_calendar() -> Command {
        Command::new("calendar")
            .about("Calendar API operations (v3, 38 methods)")
            .subcommand_required(true)
            .subcommand(group_calendar_acl())
            .subcommand(group_calendar_calendar_list())
            .subcommand(group_calendar_calendars())
            .subcommand(group_calendar_channels())
            .subcommand(group_calendar_colors())
            .subcommand(group_calendar_events())
            .subcommand(group_calendar_freebusy())
            .subcommand(group_calendar_settings())
    }

    // chat.customEmojis.create
    fn leaf_chat_custom_emojis_create() -> Command {
        Command::new("chat.customEmojis.create")
            .visible_alias("create")
            .about("Creates a custom emoji. Custom emojis are only available for Google Workspace accounts, and the...")
            .long_about("Creates a custom emoji. Custom emojis are only available for Google Workspace accounts, and the administrator must turn custom emojis on for the organization. For more information, see Learn about custom emojis in Google Chat and [Manage custom emoj")
            .args(escape_hatch_args())
    }

    // chat.customEmojis.delete
    fn leaf_chat_custom_emojis_delete() -> Command {
        Command::new("chat.customEmojis.delete")
            .visible_alias("delete")
            .about("Deletes a custom emoji. By default, users can only delete custom emoji they created. Emoji managers...")
            .long_about("Deletes a custom emoji. By default, users can only delete custom emoji they created. Emoji managers assigned by the administrator can delete any custom emoji in the organization. See [Learn about custom emojis in Google Chat](https://support.google.com")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the custom emoji to delete. Format: `customEmojis/{customEmoji}` You can use the emoji name as an alias for `{customEmoji}`. For exam"))
            .args(escape_hatch_args())
    }

    // chat.customEmojis.get
    fn leaf_chat_custom_emojis_get() -> Command {
        Command::new("chat.customEmojis.get")
            .visible_alias("get")
            .about("Returns details about a custom emoji. Custom emojis are only available for Google Workspace...")
            .long_about("Returns details about a custom emoji. Custom emojis are only available for Google Workspace accounts, and the administrator must turn custom emojis on for the organization. For more information, see Learn about custom emojis in Google Chat and [Mana")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the custom emoji. Format: `customEmojis/{customEmoji}` You can use the emoji name as an alias for `{customEmoji}`. For example, `cust"))
            .args(escape_hatch_args())
    }

    // chat.customEmojis.list
    fn leaf_chat_custom_emojis_list() -> Command {
        Command::new("chat.customEmojis.list")
            .visible_alias("list")
            .about("Lists custom emojis visible to the authenticated user. Custom emojis are only available for Google...")
            .long_about("Lists custom emojis visible to the authenticated user. Custom emojis are only available for Google Workspace accounts, and the administrator must turn custom emojis on for the organization. For more information, see [Learn about custom emojis in Google Chat](https://support.google.com/chat/answer/12")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. Supports filtering by creator. To filter by creator, you must specify a valid value. Currently only `creator(\"users/me\")` and `NOT cre"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of custom emojis returned. The service can return fewer custom emojis than this value. If unspecified, the default value is 25. The"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. (If resuming from a previous query.) A page token received from a previous list custom emoji call. Provide this to retrieve the subsequent page. When"))
            .args(escape_hatch_args())
    }

    // chat.customEmojis
    fn group_chat_custom_emojis() -> Command {
        Command::new("customEmojis")
            .about("Methods under chat.customEmojis")
            .subcommand_required(true)
            .subcommand(leaf_chat_custom_emojis_create())
            .subcommand(leaf_chat_custom_emojis_delete())
            .subcommand(leaf_chat_custom_emojis_get())
            .subcommand(leaf_chat_custom_emojis_list())
    }

    // chat.media.download
    fn leaf_chat_media_download() -> Command {
        Command::new("chat.media.download")
            .visible_alias("download")
            .about("Downloads media. Download is supported on the URI `/v1/media/{+name}?alt=media`.")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Name of the media that is being downloaded. See ReadRequest.resource_name."))
            .args(escape_hatch_args())
    }

    // chat.media.upload
    fn leaf_chat_media_upload() -> Command {
        Command::new("chat.media.upload")
            .visible_alias("upload")
            .about("Uploads an attachment. For an example, see Upload media as a file attachment. Requires user...")
            .long_about("Uploads an attachment. For an example, see Upload media as a file attachment. Requires user authentication with one of the following [authorizatio")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Resource name of the Chat space in which the attachment is uploaded. Format \"spaces/{space}\"."))
            .args(escape_hatch_args())
    }

    // chat.media
    fn group_chat_media() -> Command {
        Command::new("media")
            .about("Methods under chat.media")
            .subcommand_required(true)
            .subcommand(leaf_chat_media_download())
            .subcommand(leaf_chat_media_upload())
    }

    // chat.spaces.completeImport
    fn leaf_chat_spaces_complete_import() -> Command {
        Command::new("chat.spaces.completeImport")
            .visible_alias("completeImport")
            .about("Completes the import process for the specified space and makes it visible to users. Requires user...")
            .long_about("Completes the import process for the specified space and makes it visible to users. Requires user authentication and domain-wide delegation with the [authoriza")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the import mode space. Format: `spaces/{space}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.create
    fn leaf_chat_spaces_create() -> Command {
        Command::new("chat.spaces.create")
            .visible_alias("create")
            .about("Creates a space. Can be used to create a named space, or a group chat in `Import mode`. For an...")
            .long_about("Creates a space. Can be used to create a named space, or a group chat in `Import mode`. For an example, see Create a space. Supports the following types of [authentication](https://developers.google.com/workspace/chat/authenticate-authori")
            .arg(Arg::new("request-id").long("request-id").value_name("REQUEST_ID")
                .help("Optional. A unique ID for this request. A random UUID is recommended. Specifying a request ID makes the request idempotent, which ensures that multiple identica"))
            .args(escape_hatch_args())
    }

    // chat.spaces.delete
    fn leaf_chat_spaces_delete() -> Command {
        Command::new("chat.spaces.delete")
            .visible_alias("delete")
            .about("Deletes a named space. Always performs a cascading delete, which means that the space's child...")
            .long_about("Deletes a named space. Always performs a cascading delete, which means that the space's child resources—like messages posted in the space and memberships in the space—are also deleted. For an example, see Delete a space. Supports the foll")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the space to delete. Format: `spaces/{space}`"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.findDirectMessage
    fn leaf_chat_spaces_find_direct_message() -> Command {
        Command::new("chat.spaces.findDirectMessage")
            .visible_alias("findDirectMessage")
            .about("Returns the existing direct message with the specified user. If no direct message space is found...")
            .long_about("Returns the existing direct message with the specified user. If no direct message space is found, returns a `404 NOT_FOUND` error. For an example, see Find a direct message. With [app authentication](https://developers.google.com/workspace/chat/authe")
            .arg(Arg::new("name").long("name").value_name("NAME")
                .help("Required. Resource name of the user to find direct message with. Format: `users/{user}`, where `{user}` is either the `id` for the [person](https://developers.g"))
            .args(escape_hatch_args())
    }

    // chat.spaces.findGroupChats
    fn leaf_chat_spaces_find_group_chats() -> Command {
        Command::new("chat.spaces.findGroupChats")
            .visible_alias("findGroupChats")
            .about("Returns all spaces with `spaceType == GROUP_CHAT`, whose human memberships contain exactly the...")
            .long_about("Returns all spaces with `spaceType == GROUP_CHAT`, whose human memberships contain exactly the calling user, and the users specified in `FindGroupChatsRequest.users`. Only members that have joined the conversation are supported. For an example, see [Find group chats](https://developers.google.com/wo")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of spaces to return. The service might return fewer than this value. If unspecified, at most 10 spaces are returned. The maximum va"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous call to find group chats. Provide this parameter to retrieve the subsequent page. When paginating, all other pa"))
            .arg(Arg::new("space-view").long("space-view").value_name("SPACE_VIEW").value_parser(["SPACE_VIEW_UNSPECIFIED", "SPACE_VIEW_RESOURCE_NAME_ONLY", "SPACE_VIEW_EXPANDED"])
                .help("Requested space view type. If unset, defaults to `SPACE_VIEW_RESOURCE_NAME_ONLY`. Requests that specify `SPACE_VIEW_EXPANDED` must include scopes that allow rea"))
            .arg(Arg::new("users").long("users").value_name("USERS").action(ArgAction::Append)
                .help("Optional. Resource names of all human users in group chat with the calling user. Chat apps can't be included in the request. The maximum number of users that ca"))
            .args(escape_hatch_args())
    }

    // chat.spaces.get
    fn leaf_chat_spaces_get() -> Command {
        Command::new("chat.spaces.get")
            .visible_alias("get")
            .about("Returns details about a space. For an example, see Get details about a space. Supports the...")
            .long_about("Returns details about a space. For an example, see Get details about a space. Supports the following types of authentication: - [App authentication](https://developers.go")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the space, in the form `spaces/{space}`. Format: `spaces/{space}`"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.list
    fn leaf_chat_spaces_list() -> Command {
        Command::new("chat.spaces.list")
            .visible_alias("list")
            .about("Lists spaces the caller is a member of. Group chats and DMs aren't listed until the first message...")
            .long_about("Lists spaces the caller is a member of. Group chats and DMs aren't listed until the first message is sent. For an example, see List spaces. Supports the following types of [authentication](https://developers.google.com/workspace/chat/authen")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. You can filter spaces by the space type ([`space_type`](https://developers.google.com/workspace/chat/api/reference/rest/v1/spaces#spac"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of spaces to return. The service might return fewer than this value. If unspecified, at most 100 spaces are returned. The maximum v"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous list spaces call. Provide this parameter to retrieve the subsequent page. When paginating, the filter value sho"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members.create
    fn leaf_chat_spaces_members_create() -> Command {
        Command::new("chat.spaces.members.create")
            .visible_alias("create")
            .about("Creates a membership for the calling Chat app, a user, or a Google Group. Creating memberships for...")
            .long_about("Creates a membership for the calling Chat app, a user, or a Google Group. Creating memberships for other Chat apps isn't supported. When creating a membership, if the specified member has their auto-accept policy turned off, then they're invited, and must accept the space invitation before joining.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the space for which to create the membership. Format: spaces/{space}"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members.delete
    fn leaf_chat_spaces_members_delete() -> Command {
        Command::new("chat.spaces.members.delete")
            .visible_alias("delete")
            .about("Deletes a membership. For an example, see Remove a user or a Google Chat app from a space. Supports...")
            .long_about("Deletes a membership. For an example, see Remove a user or a Google Chat app from a space. Supports the following types of authentication: - [App authentication](http")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the membership to delete. Chat apps can delete human users' or their own memberships. Chat apps can't delete other apps' memberships."))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members.get
    fn leaf_chat_spaces_members_get() -> Command {
        Command::new("chat.spaces.members.get")
            .visible_alias("get")
            .about("Returns details about a membership. For an example, see Get details about a user's or Google Chat...")
            .long_about("Returns details about a membership. For an example, see Get details about a user's or Google Chat app's membership. Supports the following types of authentication: - [Ap")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the membership to retrieve. To get the app's own membership [by using user authentication](https://developers.google.com/workspace/ch"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members.list
    fn leaf_chat_spaces_members_list() -> Command {
        Command::new("chat.spaces.members.list")
            .visible_alias("list")
            .about("Lists memberships in a space. For an example, see List users and Google Chat apps in a space...")
            .long_about("Lists memberships in a space. For an example, see List users and Google Chat apps in a space. Listing memberships with app authentication lists memberships in")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. You can filter memberships by a member's role ([`role`](https://developers.google.com/workspace/chat/api/reference/rest/v1/spaces.memb"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of memberships to return. The service might return fewer than this value. If unspecified, at most 100 memberships are returned. The"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous call to list memberships. Provide this parameter to retrieve the subsequent page. When paginating, all other pa"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the space for which to fetch a membership list. Format: spaces/{space}"))
            .arg(Arg::new("show-groups").long("show-groups").action(ArgAction::SetTrue)
                .help("Optional. When `true`, also returns memberships associated with a Google Group, in addition to other types of memberships. If a filter is set, Google Group memb"))
            .arg(Arg::new("show-invited").long("show-invited").action(ArgAction::SetTrue)
                .help("Optional. When `true`, also returns memberships associated with invited members, in addition to other types of memberships. If a filter is set, invited membersh"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members.patch
    fn leaf_chat_spaces_members_patch() -> Command {
        Command::new("chat.spaces.members.patch")
            .visible_alias("patch")
            .about("Updates a membership. For an example, see Update a user's membership in a space. Supports the...")
            .long_about("Updates a membership. For an example, see Update a user's membership in a space. Supports the following types of authentication: - [App authentication](https://develo")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the membership, assigned by the server. Format: `spaces/{space}/members/{member}`"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The field paths to update. Separate multiple values with commas or use `*` to update all field paths. Currently supported field paths: - `role`"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.members
    fn group_chat_spaces_members() -> Command {
        Command::new("members")
            .about("Methods under chat.spaces.members")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_members_create())
            .subcommand(leaf_chat_spaces_members_delete())
            .subcommand(leaf_chat_spaces_members_get())
            .subcommand(leaf_chat_spaces_members_list())
            .subcommand(leaf_chat_spaces_members_patch())
    }

    // chat.spaces.messagePins.create
    fn leaf_chat_spaces_message_pins_create() -> Command {
        Command::new("chat.spaces.messagePins.create")
            .visible_alias("create")
            .about("Creates a message pin. Requires user authentication with one of the following authorization scopes...")
            .long_about("Creates a message pin. Requires user authentication with one of the following authorization scopes: - `https://www.googleapis.com/au")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent space in which to create the message pin. Format: spaces/{space}"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messagePins.delete
    fn leaf_chat_spaces_message_pins_delete() -> Command {
        Command::new("chat.spaces.messagePins.delete")
            .visible_alias("delete")
            .about("Deletes a message pin. Requires user authentication with one of the following authorization scopes...")
            .long_about("Deletes a message pin. Requires user authentication with one of the following authorization scopes: - `https://www.googleapis.com/au")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the message pin to remove. Format: spaces/{space}/messagePins/{message_pin}"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messagePins.list
    fn leaf_chat_spaces_message_pins_list() -> Command {
        Command::new("chat.spaces.messagePins.list")
            .visible_alias("list")
            .about("Lists message pins in a space. Users can pin important messages in spaces for easy access. For more...")
            .long_about("Lists message pins in a space. Users can pin important messages in spaces for easy access. For more information, see Pin or unpin a conversation in Google Chat. Requires [user authentication](https://developers.google.com/workspace/chat/authenticate")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of message pins returned. The service might return fewer messages than this value. The maximum value is 100. If you use a value mor"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token received from a previous list message pins call. Provide this parameter to retrieve the subsequent page. When paginating, all other param"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent space which owns the collection of pinned items Format: `spaces/{space}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messagePins
    fn group_chat_spaces_message_pins() -> Command {
        Command::new("messagePins")
            .about("Methods under chat.spaces.messagePins")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_message_pins_create())
            .subcommand(leaf_chat_spaces_message_pins_delete())
            .subcommand(leaf_chat_spaces_message_pins_list())
    }

    // chat.spaces.messages.attachments.get
    fn leaf_chat_spaces_messages_attachments_get() -> Command {
        Command::new("chat.spaces.messages.attachments.get")
            .visible_alias("get")
            .about("Gets the metadata of a message attachment. The attachment data is fetched using the media API. For...")
            .long_about("Gets the metadata of a message attachment. The attachment data is fetched using the media API. For an example, see [Get metadata about a message attachment](https://developers.google.com/workspace/chat/get-media-att")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the attachment, in the form `spaces/{space}/messages/{message}/attachments/{attachment}`."))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.attachments
    fn group_chat_spaces_messages_attachments() -> Command {
        Command::new("attachments")
            .about("Methods under chat.spaces.messages.attachments")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_messages_attachments_get())
    }

    // chat.spaces.messages.create
    fn leaf_chat_spaces_messages_create() -> Command {
        Command::new("chat.spaces.messages.create")
            .visible_alias("create")
            .about("Creates a message in a Google Chat space. For an example, see Send a message. Supports the...")
            .long_about("Creates a message in a Google Chat space. For an example, see Send a message. Supports the following types of authentication: - [App authentication](https://develope")
            .arg(Arg::new("create-message-notification-options-notification-type").long("create-message-notification-options-notification-type").value_name("CREATE_MESSAGE_NOTIFICATION_OPTIONS_NOTIFICATION_TYPE").value_parser(["NOTIFICATION_TYPE_NONE", "NOTIFICATION_TYPE_FORCE_NOTIFY", "NOTIFICATION_TYPE_SILENT"])
                .help("The notification type for the message."))
            .arg(Arg::new("message-id").long("message-id").value_name("MESSAGE_ID")
                .help("Optional. A custom ID for a message. Lets Chat apps get, update, or delete a message without needing to store the system-assigned ID in the message's resource n"))
            .arg(Arg::new("message-reply-option").long("message-reply-option").value_name("MESSAGE_REPLY_OPTION").value_parser(["MESSAGE_REPLY_OPTION_UNSPECIFIED", "REPLY_MESSAGE_FALLBACK_TO_NEW_THREAD", "REPLY_MESSAGE_OR_FAIL"])
                .help("Optional. Specifies whether a message starts a thread or replies to one. Only supported in named spaces. When [responding to user interactions](https://develope"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the space in which to create a message. Format: `spaces/{space}`"))
            .arg(Arg::new("request-id").long("request-id").value_name("REQUEST_ID")
                .help("Optional. A unique ID for this request. A random UUID is recommended. Specifying a request ID makes the request idempotent, which ensures that multiple identica"))
            .arg(Arg::new("thread-key").long("thread-key").value_name("THREAD_KEY")
                .help("Optional. Deprecated: Use thread.thread_key instead. ID for the thread. Supports up to 4000 characters. To start or add to a thread, create a message and specif"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.delete
    fn leaf_chat_spaces_messages_delete() -> Command {
        Command::new("chat.spaces.messages.delete")
            .visible_alias("delete")
            .about("Deletes a message. For an example, see Delete a message. Supports the following types of...")
            .long_about("Deletes a message. For an example, see Delete a message. Supports the following types of authentication: - [App authentication](https://developers.google.com/workspa")
            .arg(Arg::new("force").long("force").action(ArgAction::SetTrue)
                .help("Optional. When `true`, deleting a message also deletes its threaded replies. When `false`, if a message has threaded replies, deletion fails. Only applies when"))
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the message. Format: `spaces/{space}/messages/{message}` If you've set a custom ID for your message, you can use the value from the `"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.get
    fn leaf_chat_spaces_messages_get() -> Command {
        Command::new("chat.spaces.messages.get")
            .visible_alias("get")
            .about("Returns details about a message. For an example, see Get details about a message. Supports the...")
            .long_about("Returns details about a message. For an example, see Get details about a message. Supports the following types of authentication: - [App authentication](https://develop")
            .arg(Arg::new("markup-syntax").long("markup-syntax").value_name("MARKUP_SYNTAX").value_parser(["MARKUP_SYNTAX_UNSPECIFIED", "MARKUP_SYNTAX_CHAT", "MARKUP_SYNTAX_MARKDOWN"])
                .help("Optional. Specifies the desired output syntax for the Chat message `formatted_text` field."))
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the message. Format: `spaces/{space}/messages/{message}` If you've set a custom ID for your message, you can use the value from the `"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.list
    fn leaf_chat_spaces_messages_list() -> Command {
        Command::new("chat.spaces.messages.list")
            .visible_alias("list")
            .about("Lists messages in a space that the caller is a member of, including messages from blocked members...")
            .long_about("Lists messages in a space that the caller is a member of, including messages from blocked members and spaces. System messages, like those announcing new space members, aren't included. If you list messages from a space with no messages, the response is an empty object. When using a REST/HTTP interfa")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. You can filter messages by date (`create_time`) and thread (`thread.name`). To filter messages by the date they were created, specify"))
            .arg(Arg::new("markup-syntax").long("markup-syntax").value_name("MARKUP_SYNTAX").value_parser(["MARKUP_SYNTAX_UNSPECIFIED", "MARKUP_SYNTAX_CHAT", "MARKUP_SYNTAX_MARKDOWN"])
                .help("Optional. Specifies the desired output syntax for the Chat message `formatted_text` field."))
            .arg(Arg::new("order-by").long("order-by").value_name("ORDER_BY")
                .help("Optional. How the list of messages is ordered. Specify a value to order by an ordering operation. Valid ordering operation values are as follows: - `ASC` for as"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of messages returned. The service might return fewer messages than this value. If unspecified, at most 25 are returned. The maximum"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token received from a previous list messages call. Provide this parameter to retrieve the subsequent page. When paginating, all other parameter"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the space to list messages from. Format: `spaces/{space}`"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Optional. Whether to include deleted messages. Deleted messages include deleted time and metadata about their deletion, but message content is unavailable."))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.patch
    fn leaf_chat_spaces_messages_patch() -> Command {
        Command::new("chat.spaces.messages.patch")
            .visible_alias("patch")
            .about("Updates a message. There's a difference between the `patch` and `update` methods. The `patch`...")
            .long_about("Updates a message. There's a difference between the `patch` and `update` methods. The `patch` method uses a `patch` request while the `update` method uses a `put` request. We recommend using the `patch` method. For an example, see [Update a message](https://developers.google.com/workspace/chat/updat")
            .arg(Arg::new("allow-missing").long("allow-missing").action(ArgAction::SetTrue)
                .help("Optional. If `true` and the message isn't found, a new message is created and `updateMask` is ignored. The specified message ID must be [client-assigned](https:"))
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the message. Format: `spaces/{space}/messages/{message}` Where `{space}` is the ID of the space where the message is posted and `{m"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The field paths to update. Separate multiple values with commas or use `*` to update all field paths. Currently supported field paths: - `text` - `att"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.reactions.create
    fn leaf_chat_spaces_messages_reactions_create() -> Command {
        Command::new("chat.spaces.messages.reactions.create")
            .visible_alias("create")
            .about("Creates a reaction and adds it to a message. For an example, see Add a reaction to a message...")
            .long_about("Creates a reaction and adds it to a message. For an example, see Add a reaction to a message. Requires user authentication with one of the following [auth")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The message where the reaction is created. Format: `spaces/{space}/messages/{message}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.reactions.delete
    fn leaf_chat_spaces_messages_reactions_delete() -> Command {
        Command::new("chat.spaces.messages.reactions.delete")
            .visible_alias("delete")
            .about("Deletes a reaction to a message. For an example, see Delete a reaction. Requires user...")
            .long_about("Deletes a reaction to a message. For an example, see Delete a reaction. Requires user authentication with one of the following [authorization scopes](http")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Name of the reaction to delete. Format: `spaces/{space}/messages/{message}/reactions/{reaction}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.reactions.list
    fn leaf_chat_spaces_messages_reactions_list() -> Command {
        Command::new("chat.spaces.messages.reactions.list")
            .visible_alias("list")
            .about("Lists reactions to a message. For an example, see List reactions for a message. Requires user...")
            .long_about("Lists reactions to a message. For an example, see List reactions for a message. Requires user authentication with one of the following [authorization scopes")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. You can filter reactions by emoji (either `emoji.unicode`"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of reactions returned. The service can return fewer reactions than this value. If unspecified, the default value is 25. The maximum"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. (If resuming from a previous query.) A page token received from a previous list reactions call. Provide this to retrieve the subsequent page. When pag"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The message users reacted to. Format: `spaces/{space}/messages/{message}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.reactions
    fn group_chat_spaces_messages_reactions() -> Command {
        Command::new("reactions")
            .about("Methods under chat.spaces.messages.reactions")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_messages_reactions_create())
            .subcommand(leaf_chat_spaces_messages_reactions_delete())
            .subcommand(leaf_chat_spaces_messages_reactions_list())
    }

    // chat.spaces.messages.search
    fn leaf_chat_spaces_messages_search() -> Command {
        Command::new("chat.spaces.messages.search")
            .visible_alias("search")
            .about("Searches for messages in Google Chat that the calling user has access to. Returns a list of...")
            .long_about("Searches for messages in Google Chat that the calling user has access to. Returns a list of messages matching the search criteria. To search across all spaces the user has access to, set `parent` to `spaces/-`. Using any other value for `parent` results in an `INVALID_ARGUMENT` error. The returned m")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The resource name of the space to search within. To search across all spaces the user has access to, set this field to `spaces/-`. Using any other val"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages.update
    fn leaf_chat_spaces_messages_update() -> Command {
        Command::new("chat.spaces.messages.update")
            .visible_alias("update")
            .about("Updates a message. There's a difference between the `patch` and `update` methods. The `patch`...")
            .long_about("Updates a message. There's a difference between the `patch` and `update` methods. The `patch` method uses a `patch` request while the `update` method uses a `put` request. We recommend using the `patch` method. For an example, see [Update a message](https://developers.google.com/workspace/chat/updat")
            .arg(Arg::new("allow-missing").long("allow-missing").action(ArgAction::SetTrue)
                .help("Optional. If `true` and the message isn't found, a new message is created and `updateMask` is ignored. The specified message ID must be [client-assigned](https:"))
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the message. Format: `spaces/{space}/messages/{message}` Where `{space}` is the ID of the space where the message is posted and `{m"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The field paths to update. Separate multiple values with commas or use `*` to update all field paths. Currently supported field paths: - `text` - `att"))
            .args(escape_hatch_args())
    }

    // chat.spaces.messages
    fn group_chat_spaces_messages() -> Command {
        Command::new("messages")
            .about("Methods under chat.spaces.messages")
            .subcommand_required(true)
            .subcommand(group_chat_spaces_messages_attachments())
            .subcommand(leaf_chat_spaces_messages_create())
            .subcommand(leaf_chat_spaces_messages_delete())
            .subcommand(leaf_chat_spaces_messages_get())
            .subcommand(leaf_chat_spaces_messages_list())
            .subcommand(leaf_chat_spaces_messages_patch())
            .subcommand(group_chat_spaces_messages_reactions())
            .subcommand(leaf_chat_spaces_messages_search())
            .subcommand(leaf_chat_spaces_messages_update())
    }

    // chat.spaces.patch
    fn leaf_chat_spaces_patch() -> Command {
        Command::new("chat.spaces.patch")
            .visible_alias("patch")
            .about("Updates a space. For an example, see Update a space. If you're updating the `displayName` field and...")
            .long_about("Updates a space. For an example, see Update a space. If you're updating the `displayName` field and receive the error message `ALREADY_EXISTS`, try a different display name.. An existing space within the Google Workspace organization migh")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the space. Format: `spaces/{space}` Where `{space}` represents the system-assigned ID for the space. You can obtain the space ID by"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The updated field paths, comma separated if there are multiple. You can update the following fields for a space: `space_details`: Updates the space's"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("Optional. When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator wit"))
            .args(escape_hatch_args())
    }

    // chat.spaces.search
    fn leaf_chat_spaces_search() -> Command {
        Command::new("chat.spaces.search")
            .visible_alias("search")
            .about("Returns a list of spaces in a Google Workspace organization. For an example, see Search for and...")
            .long_about("Returns a list of spaces in a Google Workspace organization. For an example, see Search for and manage spaces. When `use_admin_access` is set to `false`, the results are limited to spaces where the calling user is a joined member. T")
            .arg(Arg::new("order-by").long("order-by").value_name("ORDER_BY")
                .help("Optional. How the list of spaces is ordered. Supported attributes to order by are: - `membership_count.joined_direct_human_user_count` — Denotes the count of hu"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of spaces to return. The service may return fewer than this value. If unspecified, at most 100 spaces are returned. The maximum value is 1000"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("A token, received from the previous search spaces call. Provide this parameter to retrieve the subsequent page. When paginating, all other parameters provided s"))
            .arg(Arg::new("param-query").long("param-query").value_name("PARAM_QUERY")
                .help("Discovery parameter `query`, renamed because --query is reserved here. Required. A search query. You can search by using the following parameters when `useAdminAccess` is set to `true`: - `create_time` - `customer` - `display_name`"))
            .arg(Arg::new("use-admin-access").long("use-admin-access").action(ArgAction::SetTrue)
                .help("When `true`, the method runs using the user's Google Workspace administrator privileges. The calling user must be a Google Workspace administrator with the [man"))
            .args(escape_hatch_args())
    }

    // chat.spaces.setup
    fn leaf_chat_spaces_setup() -> Command {
        Command::new("chat.spaces.setup")
            .visible_alias("setup")
            .about("Creates a space and adds specified users to it. The calling user is automatically added to the...")
            .long_about("Creates a space and adds specified users to it. The calling user is automatically added to the space, and shouldn't be specified as a membership in the request. For an example, see Set up a space with initial members. To specify the human")
            .args(escape_hatch_args())
    }

    // chat.spaces.spaceEvents.get
    fn leaf_chat_spaces_space_events_get() -> Command {
        Command::new("chat.spaces.spaceEvents.get")
            .visible_alias("get")
            .about("Returns an event from a Google Chat space. The event payload contains the most recent version of...")
            .long_about("Returns an event from a Google Chat space. The event payload contains the most recent version of the resource that changed. For example, if you request an event about a new messag")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the space event. Format: `spaces/{space}/spaceEvents/{spaceEvent}`"))
            .args(escape_hatch_args())
    }

    // chat.spaces.spaceEvents.list
    fn leaf_chat_spaces_space_events_list() -> Command {
        Command::new("chat.spaces.spaceEvents.list")
            .visible_alias("list")
            .about("Lists events from a Google Chat space. For each event, the payload contains the most recent version...")
            .long_about("Lists events from a Google Chat space. For each event, the payload contains the most recent version of the Chat resource. For example, if you list events about new space members,")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Required. A query filter. You must specify at least one event type (`event_type`) using the has `:` operator. To filter by multiple event types, use the `OR` op"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of space events returned. The service might return fewer than this value. Negative values return an `INVALID_ARGUMENT` error."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous list space events call. Provide this to retrieve the subsequent page. When paginating, all other parameters pro"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. Resource name of the Google Chat space where the events occurred. Format:"))
            .args(escape_hatch_args())
    }

    // chat.spaces.spaceEvents
    fn group_chat_spaces_space_events() -> Command {
        Command::new("spaceEvents")
            .about("Methods under chat.spaces.spaceEvents")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_space_events_get())
            .subcommand(leaf_chat_spaces_space_events_list())
    }

    // chat.spaces
    fn group_chat_spaces() -> Command {
        Command::new("spaces")
            .about("Methods under chat.spaces")
            .subcommand_required(true)
            .subcommand(leaf_chat_spaces_complete_import())
            .subcommand(leaf_chat_spaces_create())
            .subcommand(leaf_chat_spaces_delete())
            .subcommand(leaf_chat_spaces_find_direct_message())
            .subcommand(leaf_chat_spaces_find_group_chats())
            .subcommand(leaf_chat_spaces_get())
            .subcommand(leaf_chat_spaces_list())
            .subcommand(group_chat_spaces_members())
            .subcommand(group_chat_spaces_message_pins())
            .subcommand(group_chat_spaces_messages())
            .subcommand(leaf_chat_spaces_patch())
            .subcommand(leaf_chat_spaces_search())
            .subcommand(leaf_chat_spaces_setup())
            .subcommand(group_chat_spaces_space_events())
    }

    // chat.users.availability.get
    fn leaf_chat_users_availability_get() -> Command {
        Command::new("chat.users.availability.get")
            .visible_alias("get")
            .about("Returns availability information for a human user in Google Chat. For example, this can be used to...")
            .long_about("Returns availability information for a human user in Google Chat. For example, this can be used to check if a user is online or away, or to retrieve their custom status message. This method only retrieves the authenticated user's availability. Requires [user authentication](https://developers.google")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the availability to retrieve. Format: users/{user}/availability `{user}` is the id for the Person in the People API or Admin SDK"))
            .args(escape_hatch_args())
    }

    // chat.users.availability.markAsActive
    fn leaf_chat_users_availability_mark_as_active() -> Command {
        Command::new("chat.users.availability.markAsActive")
            .visible_alias("markAsActive")
            .about("Marks user as `ACTIVE` in Google Chat. Sets the user's availability state to `ACTIVE`. The `ACTIVE`...")
            .long_about("Marks user as `ACTIVE` in Google Chat. Sets the user's availability state to `ACTIVE`. The `ACTIVE` state lasts until the specified expiration, at which point the user's state becomes `AWAY`. Note that if the user is actively using Chat, the `ACTIVE` state duration may extend beyond the provided exp")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the availability to mark as active. Format: users/{user}/availability `{user}` is the id for the Person in the People API or Admi"))
            .args(escape_hatch_args())
    }

    // chat.users.availability.markAsAway
    fn leaf_chat_users_availability_mark_as_away() -> Command {
        Command::new("chat.users.availability.markAsAway")
            .visible_alias("markAsAway")
            .about("Marks user as `AWAY` in Google Chat. Sets the user's state to away and is not affected by the...")
            .long_about("Marks user as `AWAY` in Google Chat. Sets the user's state to away and is not affected by the user's activity. This method only updates the authenticated user's availability. Requires user authentication with [authoriza")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the availability to mark as away. Format: users/{user}/availability `{user}` is the id for the Person in the People API or Admin"))
            .args(escape_hatch_args())
    }

    // chat.users.availability.markAsDoNotDisturb
    fn leaf_chat_users_availability_mark_as_do_not_disturb() -> Command {
        Command::new("chat.users.availability.markAsDoNotDisturb")
            .visible_alias("markAsDoNotDisturb")
            .about("Marks user as `DO_NOT_DISTURB` in Google Chat. Sets a user's availability state to `DO_NOT_DISTURB`...")
            .long_about("Marks user as `DO_NOT_DISTURB` in Google Chat. Sets a user's availability state to `DO_NOT_DISTURB` until a specified expiration time. When in `DO_NOT_DISTURB`, users typically won't receive notifications. This method only updates the authenticated user's availability. Requires [user authentication]")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the availability to mark as Do Not Disturb. Format: users/{user}/availability `{user}` is the id for the Person in the People API"))
            .args(escape_hatch_args())
    }

    // chat.users.availability.patch
    fn leaf_chat_users_availability_patch() -> Command {
        Command::new("chat.users.availability.patch")
            .visible_alias("patch")
            .about("Updates availability information for a human user. Only the `custom_status` field can be updated...")
            .long_about("Updates availability information for a human user. Only the `custom_status` field can be updated through this method. This method only updates the authenticated user's availability. Requires user authentication with one")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the user's availability. Format: `users/{user}/availability` `{user}` is the id for the Person in the People API or Admin SDK direc"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The list of fields to update. The only field that can be updated is `custom_status`."))
            .args(escape_hatch_args())
    }

    // chat.users.availability
    fn group_chat_users_availability() -> Command {
        Command::new("availability")
            .about("Methods under chat.users.availability")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_availability_get())
            .subcommand(leaf_chat_users_availability_mark_as_active())
            .subcommand(leaf_chat_users_availability_mark_as_away())
            .subcommand(leaf_chat_users_availability_mark_as_do_not_disturb())
            .subcommand(leaf_chat_users_availability_patch())
    }

    // chat.users.sections.create
    fn leaf_chat_users_sections_create() -> Command {
        Command::new("chat.users.sections.create")
            .visible_alias("create")
            .about("Creates a section in Google Chat. Sections help users group conversations and customize the list of...")
            .long_about("Creates a section in Google Chat. Sections help users group conversations and customize the list of spaces displayed in Chat navigation panel. Only sections of type `CUSTOM_SECTION` can be created. For details, see [Create and organize sections in Google Chat](https://support.google.com/chat/answer/")
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent resource name where the section is created. Format: `users/{user}`"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.delete
    fn leaf_chat_users_sections_delete() -> Command {
        Command::new("chat.users.sections.delete")
            .visible_alias("delete")
            .about("Deletes a section of type `CUSTOM_SECTION`. If the section contains items, such as spaces, the...")
            .long_about("Deletes a section of type `CUSTOM_SECTION`. If the section contains items, such as spaces, the items are moved to Google Chat's default sections and are not deleted. For details, see Create and organize sections in Google Chat. Requires [user authen")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The name of the section to delete. Format: `users/{user}/sections/{section}`"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.items.list
    fn leaf_chat_users_sections_items_list() -> Command {
        Command::new("chat.users.sections.items.list")
            .visible_alias("list")
            .about("Lists items in a section. Only spaces can be section items. For details, see Create and organize...")
            .long_about("Lists items in a section. Only spaces can be section items. For details, see Create and organize sections in Google Chat. Requires user authentication with the [authori")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Optional. A query filter. Currently only supports filtering by space. For example, `space = spaces/{space}`. Invalid queries are rejected with an `INVALID_ARGUM"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of section items to return. The service may return fewer than this value. If unspecified, at most 10 section items will be returned"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous list section items call. Provide this to retrieve the subsequent page. When paginating, all other parameters pr"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent, which is the section resource name that owns this collection of section items. Only supports listing section items for the calling user. W"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.items.move
    fn leaf_chat_users_sections_items_move() -> Command {
        Command::new("chat.users.sections.items.move")
            .visible_alias("move")
            .about("Moves an item from one section to another. For example, if a section contains spaces, this method...")
            .long_about("Moves an item from one section to another. For example, if a section contains spaces, this method can be used to move a space to a different section. For details, see Create and organize sections in Google Chat. Requires [user authentication](https:")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the section item to move. Format: `users/{user}/sections/{section}/items/{item}`"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.items
    fn group_chat_users_sections_items() -> Command {
        Command::new("items")
            .about("Methods under chat.users.sections.items")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_sections_items_list())
            .subcommand(leaf_chat_users_sections_items_move())
    }

    // chat.users.sections.list
    fn leaf_chat_users_sections_list() -> Command {
        Command::new("chat.users.sections.list")
            .visible_alias("list")
            .about("Lists sections available to the Chat user. Sections help users group their conversations and...")
            .long_about("Lists sections available to the Chat user. Sections help users group their conversations and customize the list of spaces displayed in Chat navigation panel. For details, see Create and organize sections in Google Chat. Requires [user authentication")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of sections to return. The service may return fewer than this value. If unspecified, at most 10 sections will be returned. The maxi"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous list sections call. Provide this to retrieve the subsequent page. When paginating, all other parameters provide"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT").required(true)
                .help("Required. The parent, which is the user resource name that owns this collection of sections. Only supports listing sections for the calling user. To refer to th"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.patch
    fn leaf_chat_users_sections_patch() -> Command {
        Command::new("chat.users.sections.patch")
            .visible_alias("patch")
            .about("Updates a section. Only sections of type `CUSTOM_SECTION` can be updated. For details, see Create...")
            .long_about("Updates a section. Only sections of type `CUSTOM_SECTION` can be updated. For details, see Create and organize sections in Google Chat. Requires user authentication wit")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. Resource name of the section. For system sections, the section ID is a constant string: - DEFAULT_DIRECT_MESSAGES: `users/{user}/sections/default-di"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The mask to specify which fields to update. Currently supported field paths: - `display_name`"))
            .args(escape_hatch_args())
    }

    // chat.users.sections.position
    fn leaf_chat_users_sections_position() -> Command {
        Command::new("chat.users.sections.position")
            .visible_alias("position")
            .about("Changes the sort order of a section. For details, see Create and organize sections in Google Chat...")
            .long_about("Changes the sort order of a section. For details, see Create and organize sections in Google Chat. Requires user authentication with the [authorization scope](https://d")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. The resource name of the section to position. Format: `users/{user}/sections/{section}`"))
            .args(escape_hatch_args())
    }

    // chat.users.sections
    fn group_chat_users_sections() -> Command {
        Command::new("sections")
            .about("Methods under chat.users.sections")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_sections_create())
            .subcommand(leaf_chat_users_sections_delete())
            .subcommand(group_chat_users_sections_items())
            .subcommand(leaf_chat_users_sections_list())
            .subcommand(leaf_chat_users_sections_patch())
            .subcommand(leaf_chat_users_sections_position())
    }

    // chat.users.spaces.getSpaceReadState
    fn leaf_chat_users_spaces_get_space_read_state() -> Command {
        Command::new("chat.users.spaces.getSpaceReadState")
            .visible_alias("getSpaceReadState")
            .about("Returns details about a user's read state within a space, used to identify read and unread...")
            .long_about("Returns details about a user's read state within a space, used to identify read and unread messages. For an example, see Get details about a user's space read state. Requires [user authentication](https://developers.google.com/work")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the space read state to retrieve. Only supports getting read state for the calling user. To refer to the calling user, set one of the"))
            .args(escape_hatch_args())
    }

    // chat.users.spaces.spaceNotificationSetting.get
    fn leaf_chat_users_spaces_space_notification_setting_get() -> Command {
        Command::new("chat.users.spaces.spaceNotificationSetting.get")
            .visible_alias("get")
            .about("Gets the space notification setting. For an example, see Get the caller's space notification...")
            .long_about("Gets the space notification setting. For an example, see Get the caller's space notification setting. Requires user authentication with the")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Format: users/{user}/spaces/{space}/spaceNotificationSetting - `users/me/spaces/{space}/spaceNotificationSetting`, OR - `users/user@example.com/spaces"))
            .args(escape_hatch_args())
    }

    // chat.users.spaces.spaceNotificationSetting.patch
    fn leaf_chat_users_spaces_space_notification_setting_patch() -> Command {
        Command::new("chat.users.spaces.spaceNotificationSetting.patch")
            .visible_alias("patch")
            .about("Updates the space notification setting. For an example, see Update the caller's space notification...")
            .long_about("Updates the space notification setting. For an example, see Update the caller's space notification setting. Requires user authentication")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Identifier. The resource name of the space notification setting. Format: `users/{user}/spaces/{space}/spaceNotificationSetting`."))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. Supported field paths: - `notification_setting` - `mute_setting`"))
            .args(escape_hatch_args())
    }

    // chat.users.spaces.spaceNotificationSetting
    fn group_chat_users_spaces_space_notification_setting() -> Command {
        Command::new("spaceNotificationSetting")
            .about("Methods under chat.users.spaces.spaceNotificationSetting")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_spaces_space_notification_setting_get())
            .subcommand(leaf_chat_users_spaces_space_notification_setting_patch())
    }

    // chat.users.spaces.threads.getThreadReadState
    fn leaf_chat_users_spaces_threads_get_thread_read_state() -> Command {
        Command::new("chat.users.spaces.threads.getThreadReadState")
            .visible_alias("getThreadReadState")
            .about("Returns details about a user's read state within a thread, used to identify read and unread...")
            .long_about("Returns details about a user's read state within a thread, used to identify read and unread messages. For an example, see Get details about a user's thread read state. Requires [user authentication](https://developers.google.com/w")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Required. Resource name of the thread read state to retrieve. Only supports getting read state for the calling user. To refer to the calling user, set one of th"))
            .args(escape_hatch_args())
    }

    // chat.users.spaces.threads
    fn group_chat_users_spaces_threads() -> Command {
        Command::new("threads")
            .about("Methods under chat.users.spaces.threads")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_spaces_threads_get_thread_read_state())
    }

    // chat.users.spaces.updateSpaceReadState
    fn leaf_chat_users_spaces_update_space_read_state() -> Command {
        Command::new("chat.users.spaces.updateSpaceReadState")
            .visible_alias("updateSpaceReadState")
            .about("Updates a user's read state within a space, used to identify read and unread messages. For an...")
            .long_about("Updates a user's read state within a space, used to identify read and unread messages. For an example, see Update a user's space read state. Requires [user authentication](https://developers.google.com/workspace/chat/authenticat")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("Resource name of the space read state. Format: `users/{user}/spaces/{space}/spaceReadState`"))
            .arg(Arg::new("update-mask").long("update-mask").value_name("UPDATE_MASK")
                .help("Required. The field paths to update. Currently supported field paths: - `last_read_time` When the `last_read_time` is before the latest message create time, the"))
            .args(escape_hatch_args())
    }

    // chat.users.spaces
    fn group_chat_users_spaces() -> Command {
        Command::new("spaces")
            .about("Methods under chat.users.spaces")
            .subcommand_required(true)
            .subcommand(leaf_chat_users_spaces_get_space_read_state())
            .subcommand(group_chat_users_spaces_space_notification_setting())
            .subcommand(group_chat_users_spaces_threads())
            .subcommand(leaf_chat_users_spaces_update_space_read_state())
    }

    // chat.users
    fn group_chat_users() -> Command {
        Command::new("users")
            .about("Methods under chat.users")
            .subcommand_required(true)
            .subcommand(group_chat_users_availability())
            .subcommand(group_chat_users_sections())
            .subcommand(group_chat_users_spaces())
    }

    // chat
    fn service_chat() -> Command {
        Command::new("chat")
            .about("Google Chat API operations (v1, 54 methods)")
            .subcommand_required(true)
            .subcommand(group_chat_custom_emojis())
            .subcommand(group_chat_media())
            .subcommand(group_chat_spaces())
            .subcommand(group_chat_users())
    }

    // docs.documents.batchUpdate
    fn leaf_docs_documents_batch_update() -> Command {
        Command::new("docs.documents.batchUpdate")
            .visible_alias("batchUpdate")
            .about("Applies one or more updates to the document. Each request is validated before being applied. If any...")
            .long_about("Applies one or more updates to the document. Each request is validated before being applied. If any request is not valid, then the entire request will fail and nothing will be applied. Some requests have replies to give you some information about how they are applied. Other requests do not need to r")
            .arg(Arg::new("document-id").long("document-id").value_name("DOCUMENT_ID").required(true)
                .help("The ID of the document to update."))
            .args(escape_hatch_args())
    }

    // docs.documents.create
    fn leaf_docs_documents_create() -> Command {
        Command::new("docs.documents.create")
            .visible_alias("create")
            .about("Creates a blank document using the title given in the request. Other fields in the request...")
            .long_about("Creates a blank document using the title given in the request. Other fields in the request, including any provided content, are ignored. Returns the created document.")
            .args(escape_hatch_args())
    }

    // docs.documents.get
    fn leaf_docs_documents_get() -> Command {
        Command::new("docs.documents.get")
            .visible_alias("get")
            .about("Gets the latest version of the specified document.")
            .arg(Arg::new("comments-view-mode").long("comments-view-mode").value_name("COMMENTS_VIEW_MODE").value_parser(["COMMENTS_VIEW_MODE_UNSPECIFIED", "COMMENTS_VIEW_MODE_DEFAULT_FOR_CURRENT_ACCESS", "COMMENTS_VIEW_MODE_OMITTED", "COMMENTS_VIEW_MODE_INCLUDED"])
                .help("The comments view mode to apply to the document. This allows viewing the document with comments omitted or included. If one is not specified, COMMENTS_VIEW_MODE"))
            .arg(Arg::new("document-id").long("document-id").value_name("DOCUMENT_ID").required(true)
                .help("The ID of the document to retrieve."))
            .arg(Arg::new("include-tabs-content").long("include-tabs-content").action(ArgAction::SetTrue)
                .help("Whether to populate the `Document.tabs` field instead of the text content fields like `body` and `documentStyle` on `Document`. - When `true`: Document content"))
            .arg(Arg::new("suggestions-view-mode").long("suggestions-view-mode").value_name("SUGGESTIONS_VIEW_MODE").value_parser(["DEFAULT_FOR_CURRENT_ACCESS", "SUGGESTIONS_INLINE", "PREVIEW_SUGGESTIONS_ACCEPTED", "PREVIEW_WITHOUT_SUGGESTIONS"])
                .help("The suggestions view mode to apply to the document. This allows viewing the document with all suggestions inline, accepted or rejected. If one is not specified,"))
            .args(escape_hatch_args())
    }

    // docs.documents
    fn group_docs_documents() -> Command {
        Command::new("documents")
            .about("Methods under docs.documents")
            .subcommand_required(true)
            .subcommand(leaf_docs_documents_batch_update())
            .subcommand(leaf_docs_documents_create())
            .subcommand(leaf_docs_documents_get())
    }

    // docs
    fn service_docs() -> Command {
        Command::new("docs")
            .about("Google Docs API operations (v1, 3 methods)")
            .subcommand_required(true)
            .subcommand(group_docs_documents())
    }

    // drive.about.get
    fn leaf_drive_about_get() -> Command {
        Command::new("drive.about.get")
            .visible_alias("get")
            .about("Gets information about the user, the user's Drive, and system capabilities. For more information...")
            .long_about("Gets information about the user, the user's Drive, and system capabilities. For more information, see Return user info. Required: The `fields` parameter must be set. To return the exact fields you need, see [Return specific fields")
            .args(escape_hatch_args())
    }

    // drive.about
    fn group_drive_about() -> Command {
        Command::new("about")
            .about("Methods under drive.about")
            .subcommand_required(true)
            .subcommand(leaf_drive_about_get())
    }

    // drive.accessproposals.get
    fn leaf_drive_accessproposals_get() -> Command {
        Command::new("drive.accessproposals.get")
            .visible_alias("get")
            .about("Retrieves an access proposal by ID. For more information, see Manage pending access proposals.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the item the request is on."))
            .arg(Arg::new("proposal-id").long("proposal-id").value_name("PROPOSAL_ID").required(true)
                .help("Required. The ID of the access proposal to resolve."))
            .args(escape_hatch_args())
    }

    // drive.accessproposals.list
    fn leaf_drive_accessproposals_list() -> Command {
        Command::new("drive.accessproposals.list")
            .visible_alias("list")
            .about("List the access proposals on a file. For more information, see Manage pending access proposals...")
            .long_about("List the access proposals on a file. For more information, see Manage pending access proposals. Note: Only approvers are able to list access proposals on a file. If the user isn't an approver, a 403 error is returned.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the item the request is on."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of results per page."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. The continuation token on the list of access requests."))
            .args(escape_hatch_args())
    }

    // drive.accessproposals.resolve
    fn leaf_drive_accessproposals_resolve() -> Command {
        Command::new("drive.accessproposals.resolve")
            .visible_alias("resolve")
            .about("Approves or denies an access proposal. For more information, see Manage pending access proposals.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the item the request is on."))
            .arg(Arg::new("proposal-id").long("proposal-id").value_name("PROPOSAL_ID").required(true)
                .help("Required. The ID of the access proposal to resolve."))
            .args(escape_hatch_args())
    }

    // drive.accessproposals
    fn group_drive_accessproposals() -> Command {
        Command::new("accessproposals")
            .about("Methods under drive.accessproposals")
            .subcommand_required(true)
            .subcommand(leaf_drive_accessproposals_get())
            .subcommand(leaf_drive_accessproposals_list())
            .subcommand(leaf_drive_accessproposals_resolve())
    }

    // drive.approvals.approve
    fn leaf_drive_approvals_approve() -> Command {
        Command::new("drive.approvals.approve")
            .visible_alias("approve")
            .about("Approves an approval. For more information, see Manage approvals. This is used to update the...")
            .long_about("Approves an approval. For more information, see Manage approvals. This is used to update the ReviewerResponse of the requesting user with a Response of `APPROVED`. If this is the last required reviewer response, this also complete")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval to approve."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.cancel
    fn leaf_drive_approvals_cancel() -> Command {
        Command::new("drive.approvals.cancel")
            .visible_alias("cancel")
            .about("Cancels an approval. For more information, see Manage approvals. Updates the approval Status to...")
            .long_about("Cancels an approval. For more information, see Manage approvals. Updates the approval Status to `CANCELLED`. This can be called by any user with the `writer` permission on the file while the approval Status is `IN_PROGRESS`.")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval to cancel."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.comment
    fn leaf_drive_approvals_comment() -> Command {
        Command::new("drive.approvals.comment")
            .visible_alias("comment")
            .about("Comments on an approval. For more information, see Manage approvals. This sends a notification to...")
            .long_about("Comments on an approval. For more information, see Manage approvals. This sends a notification to both the initiator and the reviewers. Additionally, a message is also added to the approval activity log.")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval to comment on."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.decline
    fn leaf_drive_approvals_decline() -> Command {
        Command::new("drive.approvals.decline")
            .visible_alias("decline")
            .about("Declines an approval. For more information, see Manage approvals. This is used to update the...")
            .long_about("Declines an approval. For more information, see Manage approvals. This is used to update the ReviewerResponse of the requesting user with a Response of `DECLINED`. This also completes the approval and sets the approval Status to `")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval to decline."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.get
    fn leaf_drive_approvals_get() -> Command {
        Command::new("drive.approvals.get")
            .visible_alias("get")
            .about("Gets an approval by ID. For more information, see Manage approvals.")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.list
    fn leaf_drive_approvals_list() -> Command {
        Command::new("drive.approvals.list")
            .visible_alias("list")
            .about("Lists the approvals on a file. For more information, see Manage approvals. By default, this method...")
            .long_about("Lists the approvals on a file. For more information, see Manage approvals. By default, this method returns a minimal response that may not include the items array. To retrieve approval details, you must explicitly specify the fiel")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of approvals to return. When not set, at most 100 approvals are returned."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from a previous response."))
            .args(escape_hatch_args())
    }

    // drive.approvals.reassign
    fn leaf_drive_approvals_reassign() -> Command {
        Command::new("drive.approvals.reassign")
            .visible_alias("reassign")
            .about("Reassigns the reviewers on an approval. For more information, see Manage approvals. Adds or...")
            .long_about("Reassigns the reviewers on an approval. For more information, see Manage approvals. Adds or replaces reviewers in the ReviewerResponse of the approval. This can be called by any user with the `writer` permission on the file while")
            .arg(Arg::new("approval-id").long("approval-id").value_name("APPROVAL_ID").required(true)
                .help("Required. The ID of the approval to reassign."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is on."))
            .args(escape_hatch_args())
    }

    // drive.approvals.start
    fn leaf_drive_approvals_start() -> Command {
        Command::new("drive.approvals.start")
            .visible_alias("start")
            .about("Starts an approval on a file. For more information, see Manage approvals.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file that the approval is created on."))
            .args(escape_hatch_args())
    }

    // drive.approvals
    fn group_drive_approvals() -> Command {
        Command::new("approvals")
            .about("Methods under drive.approvals")
            .subcommand_required(true)
            .subcommand(leaf_drive_approvals_approve())
            .subcommand(leaf_drive_approvals_cancel())
            .subcommand(leaf_drive_approvals_comment())
            .subcommand(leaf_drive_approvals_decline())
            .subcommand(leaf_drive_approvals_get())
            .subcommand(leaf_drive_approvals_list())
            .subcommand(leaf_drive_approvals_reassign())
            .subcommand(leaf_drive_approvals_start())
    }

    // drive.apps.get
    fn leaf_drive_apps_get() -> Command {
        Command::new("drive.apps.get")
            .visible_alias("get")
            .about("Gets a specific app. For more information, see Return user info.")
            .arg(Arg::new("app-id").long("app-id").value_name("APP_ID").required(true)
                .help("The ID of the app."))
            .args(escape_hatch_args())
    }

    // drive.apps.list
    fn leaf_drive_apps_list() -> Command {
        Command::new("drive.apps.list")
            .visible_alias("list")
            .about("Lists a user's installed apps. For more information, see Return user info.")
            .arg(Arg::new("app-filter-extensions").long("app-filter-extensions").value_name("APP_FILTER_EXTENSIONS")
                .help("A comma-separated list of file extensions to limit returned results. All results within the given app query scope which can open any of the given file extension"))
            .arg(Arg::new("app-filter-mime-types").long("app-filter-mime-types").value_name("APP_FILTER_MIME_TYPES")
                .help("A comma-separated list of file extensions to limit returned results. All results within the given app query scope which can open any of the given MIME types wil"))
            .arg(Arg::new("language-code").long("language-code").value_name("LANGUAGE_CODE")
                .help("A language or locale code, as defined by BCP 47, with some extensions from Unicode's LDML format (http://www.unicode.org/reports/tr35/)."))
            .args(escape_hatch_args())
    }

    // drive.apps
    fn group_drive_apps() -> Command {
        Command::new("apps")
            .about("Methods under drive.apps")
            .subcommand_required(true)
            .subcommand(leaf_drive_apps_get())
            .subcommand(leaf_drive_apps_list())
    }

    // drive.changes.getStartPageToken
    fn leaf_drive_changes_get_start_page_token() -> Command {
        Command::new("drive.changes.getStartPageToken")
            .visible_alias("getStartPageToken")
            .about("Gets the starting pageToken for listing future changes. For more information, see Retrieve changes.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID")
                .help("The ID of the shared drive for which the starting pageToken for listing future changes from that shared drive will be returned."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID")
                .help("Deprecated: Use `driveId` instead."))
            .args(escape_hatch_args())
    }

    // drive.changes.list
    fn leaf_drive_changes_list() -> Command {
        Command::new("drive.changes.list")
            .visible_alias("list")
            .about("Lists the changes for a user or shared drive. For more information, see Retrieve changes.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID")
                .help("The shared drive from which changes will be returned. If specified the change IDs will be reflective of the shared drive; use the combined drive ID and change I"))
            .arg(Arg::new("include-corpus-removals").long("include-corpus-removals").action(ArgAction::SetTrue)
                .help("Whether changes should include the file resource if the file is still accessible by the user at the time of the request, even when a file was removed from the l"))
            .arg(Arg::new("include-items-from-all-drives").long("include-items-from-all-drives").action(ArgAction::SetTrue)
                .help("Whether both My Drive and shared drive items should be included in results."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only 'published' is supported."))
            .arg(Arg::new("include-removed").long("include-removed").action(ArgAction::SetTrue)
                .help("Whether to include changes indicating that items have been removed from the list of changes, for example by deletion or loss of access."))
            .arg(Arg::new("include-team-drive-items").long("include-team-drive-items").action(ArgAction::SetTrue)
                .help("Deprecated: Use `includeItemsFromAllDrives` instead."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of changes to return. The service may return fewer than this value. If unspecified, at most 100 changes will be returned. The maximum value i"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN").required(true)
                .help("The token for continuing a previous list request on the next page. This should be set to the value of 'nextPageToken' from the previous response or to the respo"))
            .arg(Arg::new("restrict-to-my-drive").long("restrict-to-my-drive").action(ArgAction::SetTrue)
                .help("Whether to restrict the results to changes inside the My Drive hierarchy. This omits changes to files such as those in the Application Data folder or shared fil"))
            .arg(Arg::new("spaces").long("spaces").value_name("SPACES")
                .help("A comma-separated list of spaces to query within the corpora. Supported values are 'drive' and 'appDataFolder'."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID")
                .help("Deprecated: Use `driveId` instead."))
            .args(escape_hatch_args())
    }

    // drive.changes.watch
    fn leaf_drive_changes_watch() -> Command {
        Command::new("drive.changes.watch")
            .visible_alias("watch")
            .about("Subscribes to changes for a user. For more information, see Notifications for resource changes.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID")
                .help("The shared drive from which changes will be returned. If specified the change IDs will be reflective of the shared drive; use the combined drive ID and change I"))
            .arg(Arg::new("include-corpus-removals").long("include-corpus-removals").action(ArgAction::SetTrue)
                .help("Whether changes should include the file resource if the file is still accessible by the user at the time of the request, even when a file was removed from the l"))
            .arg(Arg::new("include-items-from-all-drives").long("include-items-from-all-drives").action(ArgAction::SetTrue)
                .help("Whether both My Drive and shared drive items should be included in results."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only 'published' is supported."))
            .arg(Arg::new("include-removed").long("include-removed").action(ArgAction::SetTrue)
                .help("Whether to include changes indicating that items have been removed from the list of changes, for example by deletion or loss of access."))
            .arg(Arg::new("include-team-drive-items").long("include-team-drive-items").action(ArgAction::SetTrue)
                .help("Deprecated: Use `includeItemsFromAllDrives` instead."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of changes to return. The service may return fewer than this value. If unspecified, at most 100 changes will be returned. The maximum value i"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN").required(true)
                .help("The token for continuing a previous list request on the next page. This should be set to the value of 'nextPageToken' from the previous response or to the respo"))
            .arg(Arg::new("restrict-to-my-drive").long("restrict-to-my-drive").action(ArgAction::SetTrue)
                .help("Whether to restrict the results to changes inside the My Drive hierarchy. This omits changes to files such as those in the Application Data folder or shared fil"))
            .arg(Arg::new("spaces").long("spaces").value_name("SPACES")
                .help("A comma-separated list of spaces to query within the corpora. Supported values are 'drive' and 'appDataFolder'."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID")
                .help("Deprecated: Use `driveId` instead."))
            .args(escape_hatch_args())
    }

    // drive.changes
    fn group_drive_changes() -> Command {
        Command::new("changes")
            .about("Methods under drive.changes")
            .subcommand_required(true)
            .subcommand(leaf_drive_changes_get_start_page_token())
            .subcommand(leaf_drive_changes_list())
            .subcommand(leaf_drive_changes_watch())
    }

    // drive.channels.stop
    fn leaf_drive_channels_stop() -> Command {
        Command::new("drive.channels.stop")
            .visible_alias("stop")
            .about("Stops watching resources through this channel. For more information, see Notifications for resource...")
            .long_about("Stops watching resources through this channel. For more information, see Notifications for resource changes.")
            .args(escape_hatch_args())
    }

    // drive.channels
    fn group_drive_channels() -> Command {
        Command::new("channels")
            .about("Methods under drive.channels")
            .subcommand_required(true)
            .subcommand(leaf_drive_channels_stop())
    }

    // drive.comments.create
    fn leaf_drive_comments_create() -> Command {
        Command::new("drive.comments.create")
            .visible_alias("create")
            .about("Creates a comment on a file. For more information, see Manage comments and replies. Required: The...")
            .long_about("Creates a comment on a file. For more information, see Manage comments and replies. Required: The `fields` parameter must be set. To return the exact fields you need, see [Return specific fields](https://developers.google.co")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .args(escape_hatch_args())
    }

    // drive.comments.delete
    fn leaf_drive_comments_delete() -> Command {
        Command::new("drive.comments.delete")
            .visible_alias("delete")
            .about("Deletes a comment. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .args(escape_hatch_args())
    }

    // drive.comments.get
    fn leaf_drive_comments_get() -> Command {
        Command::new("drive.comments.get")
            .visible_alias("get")
            .about("Gets a comment by ID. For more information, see Manage comments and replies. Required: The `fields`...")
            .long_about("Gets a comment by ID. For more information, see Manage comments and replies. Required: The `fields` parameter must be set. To return the exact fields you need, see [Return specific fields](https://developers.google.com/works")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-deleted").long("include-deleted").action(ArgAction::SetTrue)
                .help("Whether to return deleted comments. Deleted comments will not include their original content."))
            .args(escape_hatch_args())
    }

    // drive.comments.list
    fn leaf_drive_comments_list() -> Command {
        Command::new("drive.comments.list")
            .visible_alias("list")
            .about("Lists a file's comments. For more information, see Manage comments and replies. Required: The...")
            .long_about("Lists a file's comments. For more information, see Manage comments and replies. Required: The `fields` parameter must be set. To return the exact fields you need, see [Return specific fields](https://developers.google.com/wo")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-deleted").long("include-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted comments. Deleted comments will not include their original content."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of comments to return. The service may return fewer than this value. If unspecified, at most 20 comments will be returned. The maximum value"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of 'nextPageToken' from the previous response."))
            .arg(Arg::new("start-modified-time").long("start-modified-time").value_name("START_MODIFIED_TIME")
                .help("The minimum value of 'modifiedTime' for the result comments (RFC 3339 date-time)."))
            .args(escape_hatch_args())
    }

    // drive.comments.update
    fn leaf_drive_comments_update() -> Command {
        Command::new("drive.comments.update")
            .visible_alias("update")
            .about("Updates a comment with patch semantics. For more information, see Manage comments and replies...")
            .long_about("Updates a comment with patch semantics. For more information, see Manage comments and replies. Required: The `fields` parameter must be set. To return the exact fields you need, see [Return specific fields](https://developer")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .args(escape_hatch_args())
    }

    // drive.comments
    fn group_drive_comments() -> Command {
        Command::new("comments")
            .about("Methods under drive.comments")
            .subcommand_required(true)
            .subcommand(leaf_drive_comments_create())
            .subcommand(leaf_drive_comments_delete())
            .subcommand(leaf_drive_comments_get())
            .subcommand(leaf_drive_comments_list())
            .subcommand(leaf_drive_comments_update())
    }

    // drive.drives.create
    fn leaf_drive_drives_create() -> Command {
        Command::new("drive.drives.create")
            .visible_alias("create")
            .about("Creates a shared drive. For more information, see Manage shared drives.")
            .arg(Arg::new("request-id").long("request-id").value_name("REQUEST_ID").required(true)
                .help("Required. An ID, such as a random UUID, which uniquely identifies this user's request for idempotent creation of a shared drive. A repeated request by the same"))
            .args(escape_hatch_args())
    }

    // drive.drives.delete
    fn leaf_drive_drives_delete() -> Command {
        Command::new("drive.drives.delete")
            .visible_alias("delete")
            .about("Permanently deletes a shared drive for which the user is an `organizer`. The shared drive cannot...")
            .long_about("Permanently deletes a shared drive for which the user is an `organizer`. The shared drive cannot contain any untrashed items. For more information, see Manage shared drives.")
            .arg(Arg::new("allow-item-deletion").long("allow-item-deletion").action(ArgAction::SetTrue)
                .help("Whether any items inside the shared drive should also be deleted. This option is only supported when `useDomainAdminAccess` is also set to `true`."))
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID").required(true)
                .help("The ID of the shared drive."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then the requester will be granted access if they are an administrator of the domain to which the s"))
            .args(escape_hatch_args())
    }

    // drive.drives.get
    fn leaf_drive_drives_get() -> Command {
        Command::new("drive.drives.get")
            .visible_alias("get")
            .about("Gets a shared drive's metadata by ID. For more information, see Manage shared drives.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID").required(true)
                .help("The ID of the shared drive."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then the requester will be granted access if they are an administrator of the domain to which the s"))
            .args(escape_hatch_args())
    }

    // drive.drives.hide
    fn leaf_drive_drives_hide() -> Command {
        Command::new("drive.drives.hide")
            .visible_alias("hide")
            .about("Hides a shared drive from the default view. For more information, see Manage shared drives.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID").required(true)
                .help("The ID of the shared drive."))
            .args(escape_hatch_args())
    }

    // drive.drives.list
    fn leaf_drive_drives_list() -> Command {
        Command::new("drive.drives.list")
            .visible_alias("list")
            .about("Lists the user's shared drives. This method accepts the `q` parameter, which is a search query...")
            .long_about("Lists the user's shared drives. This method accepts the `q` parameter, which is a search query combining one or more search terms. For more information, see the Search for shared drives guide.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of shared drives to return. The service may return fewer than this value. If unspecified, at most 10 shared drives will be returned. The maxi"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token for shared drives."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Query string for searching shared drives."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then all shared drives of the domain in which the requester is an administrator are returned."))
            .args(escape_hatch_args())
    }

    // drive.drives.unhide
    fn leaf_drive_drives_unhide() -> Command {
        Command::new("drive.drives.unhide")
            .visible_alias("unhide")
            .about("Restores a shared drive to the default view. For more information, see Manage shared drives.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID").required(true)
                .help("The ID of the shared drive."))
            .args(escape_hatch_args())
    }

    // drive.drives.update
    fn leaf_drive_drives_update() -> Command {
        Command::new("drive.drives.update")
            .visible_alias("update")
            .about("Updates the metadata for a shared drive. For more information, see Manage shared drives.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID").required(true)
                .help("The ID of the shared drive."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then the requester will be granted access if they are an administrator of the domain to which the s"))
            .args(escape_hatch_args())
    }

    // drive.drives
    fn group_drive_drives() -> Command {
        Command::new("drives")
            .about("Methods under drive.drives")
            .subcommand_required(true)
            .subcommand(leaf_drive_drives_create())
            .subcommand(leaf_drive_drives_delete())
            .subcommand(leaf_drive_drives_get())
            .subcommand(leaf_drive_drives_hide())
            .subcommand(leaf_drive_drives_list())
            .subcommand(leaf_drive_drives_unhide())
            .subcommand(leaf_drive_drives_update())
    }

    // drive.files.copy
    fn leaf_drive_files_copy() -> Command {
        Command::new("drive.files.copy")
            .visible_alias("copy")
            .about("Creates a copy of a file and applies any requested updates with patch semantics. For more...")
            .long_about("Creates a copy of a file and applies any requested updates with patch semantics. For more information, see Create and manage files.")
            .arg(Arg::new("copy-comments").long("copy-comments").action(ArgAction::SetTrue)
                .help("Whether to copy the open (unresolved) comments associated with the file."))
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: Copying files into multiple folders is no longer supported. Use shortcuts instead."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("ignore-default-visibility").long("ignore-default-visibility").action(ArgAction::SetTrue)
                .help("Whether to ignore the domain's default visibility settings for the created file. Domain administrators can choose to make all uploaded files visible to the doma"))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("keep-revision-forever").long("keep-revision-forever").action(ArgAction::SetTrue)
                .help("Whether to set the `keepForever` field in the new head revision. This is only applicable to files with binary content in Google Drive. Only 200 revisions for th"))
            .arg(Arg::new("ocr-language").long("ocr-language").value_name("OCR_LANGUAGE")
                .help("A language hint for OCR processing during image import (ISO 639-1 code)."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .args(escape_hatch_args())
    }

    // drive.files.create
    fn leaf_drive_files_create() -> Command {
        Command::new("drive.files.create")
            .visible_alias("create")
            .about("Creates a file. For more information, see Create and manage files. This method supports an...")
            .long_about("Creates a file. For more information, see Create and manage files. This method supports an */upload* URI and accepts uploaded media with the following characteristics: - *Maximum file size:* 5,120 GB - *Accepted Media MIME types")
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: Creating files in multiple folders is no longer supported."))
            .arg(Arg::new("ignore-default-visibility").long("ignore-default-visibility").action(ArgAction::SetTrue)
                .help("Whether to ignore the domain's default visibility settings for the created file. Domain administrators can choose to make all uploaded files visible to the doma"))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("keep-revision-forever").long("keep-revision-forever").action(ArgAction::SetTrue)
                .help("Whether to set the `keepForever` field in the new head revision. This is only applicable to files with binary content in Google Drive. Only 200 revisions for th"))
            .arg(Arg::new("ocr-language").long("ocr-language").value_name("OCR_LANGUAGE")
                .help("A language hint for OCR processing during image import (ISO 639-1 code)."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("use-content-as-indexable-text").long("use-content-as-indexable-text").action(ArgAction::SetTrue)
                .help("Whether to use the uploaded content as indexable text."))
            .args(escape_hatch_args())
    }

    // drive.files.delete
    fn leaf_drive_files_delete() -> Command {
        Command::new("drive.files.delete")
            .visible_alias("delete")
            .about("Permanently deletes a file owned by the user without moving it to the trash. For more information...")
            .long_about("Permanently deletes a file owned by the user without moving it to the trash. For more information, see Trash or delete files and folders. If the file belongs to a shared drive, the user must be an `organizer` on the parent folder. If")
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: If an item isn't in a shared drive and its last parent is deleted but the item itself isn't, the item will be placed under its owner's root."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .args(escape_hatch_args())
    }

    // drive.files.download
    fn leaf_drive_files_download() -> Command {
        Command::new("drive.files.download")
            .visible_alias("download")
            .about("Downloads the content of a file. For more information, see Download and export files. Operations...")
            .long_about("Downloads the content of a file. For more information, see Download and export files. Operations are valid for 24 hours from the time of creation.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("Required. The ID of the file to download."))
            .arg(Arg::new("mime-type").long("mime-type").value_name("MIME_TYPE")
                .help("Optional. The MIME type the file should be downloaded as. This field can only be set when downloading Google Workspace documents. For a list of supported MIME t"))
            .arg(Arg::new("revision-id").long("revision-id").value_name("REVISION_ID")
                .help("Optional. The revision ID of the file to download. This field can only be set when downloading blob files, Google Docs, and Google Sheets. Returns `INVALID_ARGU"))
            .args(escape_hatch_args())
    }

    // drive.files.emptyTrash
    fn leaf_drive_files_empty_trash() -> Command {
        Command::new("drive.files.emptyTrash")
            .visible_alias("emptyTrash")
            .about("Permanently deletes all of the user's trashed files. For more information, see Trash or delete...")
            .long_about("Permanently deletes all of the user's trashed files. For more information, see Trash or delete files and folders.")
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID")
                .help("If set, empties the trash of the provided shared drive."))
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: If an item isn't in a shared drive and its last parent is deleted but the item itself isn't, the item will be placed under its owner's root."))
            .args(escape_hatch_args())
    }

    // drive.files.export
    fn leaf_drive_files_export() -> Command {
        Command::new("drive.files.export")
            .visible_alias("export")
            .about("Exports a Google Workspace document to the requested MIME type and returns exported byte content...")
            .long_about("Exports a Google Workspace document to the requested MIME type and returns exported byte content. For more information, see Download and export files. Note that the exported content is limited to 10 MB.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("mime-type").long("mime-type").value_name("MIME_TYPE").required(true)
                .help("Required. The MIME type of the format requested for this export. For a list of supported MIME types, see [Export MIME types for Google Workspace documents](/wor"))
            .args(escape_hatch_args())
    }

    // drive.files.generateCseToken
    fn leaf_drive_files_generate_cse_token() -> Command {
        Command::new("drive.files.generateCseToken")
            .visible_alias("generateCseToken")
            .about("Generates a CSE token which can be used to create or update CSE files.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID")
                .help("The ID of the file for which the JWT should be generated. If not provided, an id will be generated."))
            .arg(Arg::new("parent").long("parent").value_name("PARENT")
                .help("The ID of the expected parent of the file. Used when generating a JWT for a new CSE file. If specified, the parent will be fetched, and if the parent is a share"))
            .args(escape_hatch_args())
    }

    // drive.files.generateIds
    fn leaf_drive_files_generate_ids() -> Command {
        Command::new("drive.files.generateIds")
            .visible_alias("generateIds")
            .about("Generates a set of file IDs which can be provided in create or copy requests. For more information...")
            .long_about("Generates a set of file IDs which can be provided in create or copy requests. For more information, see Create and manage files.")
            .arg(Arg::new("count").long("count").value_name("COUNT").value_parser(clap::value_parser!(i64))
                .help("The number of IDs to return."))
            .arg(Arg::new("space").long("space").value_name("SPACE")
                .help("The space in which the IDs can be used to create files. Supported values are `drive` and `appDataFolder`. (Default: `drive`.) For more information, see [File or"))
            .arg(Arg::new("type").long("type").value_name("TYPE")
                .help("The type of items which the IDs can be used for. Supported values are `files` and `shortcuts`. Note that `shortcuts` are only supported in the `drive` `space`."))
            .args(escape_hatch_args())
    }

    // drive.files.get
    fn leaf_drive_files_get() -> Command {
        Command::new("drive.files.get")
            .visible_alias("get")
            .about("Gets a file's metadata or content by ID. For more information, see Search for files and folders. If...")
            .long_about("Gets a file's metadata or content by ID. For more information, see Search for files and folders. If you provide the URL parameter `alt=media`, then the response includes the file contents in the response body. Downloading conte")
            .arg(Arg::new("acknowledge-abuse").long("acknowledge-abuse").action(ArgAction::SetTrue)
                .help("Whether the user is acknowledging the risk of downloading known malware or other abusive files. This is only applicable when the `alt` parameter is set to `medi"))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .args(escape_hatch_args())
    }

    // drive.files.list
    fn leaf_drive_files_list() -> Command {
        Command::new("drive.files.list")
            .visible_alias("list")
            .about("Lists the user's files. For more information, see Search for files and folders. This method accepts...")
            .long_about("Lists the user's files. For more information, see Search for files and folders. This method accepts the `q` parameter, which is a search query combining one or more search terms. This method returns *all* files by default, incl")
            .arg(Arg::new("corpora").long("corpora").value_name("CORPORA")
                .help("Specifies a collection of items (files or documents) to which the query applies. Supported items include: * `user` * `domain` * `drive` * `allDrives` Prefer `us"))
            .arg(Arg::new("corpus").long("corpus").value_name("CORPUS").value_parser(["domain", "user"])
                .help("Deprecated: The source of files to list. Use `corpora` instead."))
            .arg(Arg::new("drive-id").long("drive-id").value_name("DRIVE_ID")
                .help("ID of the shared drive to search."))
            .arg(Arg::new("include-items-from-all-drives").long("include-items-from-all-drives").action(ArgAction::SetTrue)
                .help("Whether both My Drive and shared drive items should be included in results."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("include-team-drive-items").long("include-team-drive-items").action(ArgAction::SetTrue)
                .help("Deprecated: Use `includeItemsFromAllDrives` instead."))
            .arg(Arg::new("order-by").long("order-by").value_name("ORDER_BY")
                .help("A comma-separated list of sort keys. Valid keys are: * `createdTime`: When the file was created. Avoid using this key for queries on large item collections as i"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of files to return. The service may return fewer than this value. If unspecified, at most 100 files will be returned for shared drives, and t"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from the previous response."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("A query for filtering the file results. For supported syntax, see Search for files and folders."))
            .arg(Arg::new("spaces").long("spaces").value_name("SPACES")
                .help("A comma-separated list of spaces to query within the corpora. Supported values are `drive` and `appDataFolder`. For more information, see [File organization](ht"))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID")
                .help("Deprecated: Use `driveId` instead."))
            .args(escape_hatch_args())
    }

    // drive.files.listLabels
    fn leaf_drive_files_list_labels() -> Command {
        Command::new("drive.files.listLabels")
            .visible_alias("listLabels")
            .about("Lists the labels on a file. For more information, see List labels on a file.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID for the file."))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("The maximum number of labels to return per page. When not set, defaults to 100."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from the previous response."))
            .args(escape_hatch_args())
    }

    // drive.files.modifyLabels
    fn leaf_drive_files_modify_labels() -> Command {
        Command::new("drive.files.modifyLabels")
            .visible_alias("modifyLabels")
            .about("Modifies the set of labels applied to a file. For more information, see Set a label field on a...")
            .long_about("Modifies the set of labels applied to a file. For more information, see Set a label field on a file. Returns a list of the labels that were added or modified.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file to which the labels belong."))
            .args(escape_hatch_args())
    }

    // drive.files.update
    fn leaf_drive_files_update() -> Command {
        Command::new("drive.files.update")
            .visible_alias("update")
            .about("Updates a file's metadata, content, or both. When calling this method, only populate fields in the...")
            .long_about("Updates a file's metadata, content, or both. When calling this method, only populate fields in the request that you want to modify. When updating fields, some fields might be changed automatically, such as `modifiedDate`. This method supports patch semantics. This method supports an */upload* URI an")
            .arg(Arg::new("add-parents").long("add-parents").value_name("ADD_PARENTS")
                .help("A comma-separated list of parent IDs to add."))
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: Adding files to multiple folders is no longer supported. Use shortcuts instead."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("keep-revision-forever").long("keep-revision-forever").action(ArgAction::SetTrue)
                .help("Whether to set the `keepForever` field in the new head revision. This is only applicable to files with binary content in Google Drive. Only 200 revisions for th"))
            .arg(Arg::new("ocr-language").long("ocr-language").value_name("OCR_LANGUAGE")
                .help("A language hint for OCR processing during image import (ISO 639-1 code)."))
            .arg(Arg::new("remove-parents").long("remove-parents").value_name("REMOVE_PARENTS")
                .help("A comma-separated list of parent IDs to remove."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("use-content-as-indexable-text").long("use-content-as-indexable-text").action(ArgAction::SetTrue)
                .help("Whether to use the uploaded content as indexable text."))
            .args(escape_hatch_args())
    }

    // drive.files.watch
    fn leaf_drive_files_watch() -> Command {
        Command::new("drive.files.watch")
            .visible_alias("watch")
            .about("Subscribes to changes to a file. For more information, see Notifications for resource changes.")
            .arg(Arg::new("acknowledge-abuse").long("acknowledge-abuse").action(ArgAction::SetTrue)
                .help("Whether the user is acknowledging the risk of downloading known malware or other abusive files. This is only applicable when the `alt` parameter is set to `medi"))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-labels").long("include-labels").value_name("INCLUDE_LABELS")
                .help("A comma-separated list of IDs of labels to include in the `labelInfo` part of the response."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .args(escape_hatch_args())
    }

    // drive.files
    fn group_drive_files() -> Command {
        Command::new("files")
            .about("Methods under drive.files")
            .subcommand_required(true)
            .subcommand(leaf_drive_files_copy())
            .subcommand(leaf_drive_files_create())
            .subcommand(leaf_drive_files_delete())
            .subcommand(leaf_drive_files_download())
            .subcommand(leaf_drive_files_empty_trash())
            .subcommand(leaf_drive_files_export())
            .subcommand(leaf_drive_files_generate_cse_token())
            .subcommand(leaf_drive_files_generate_ids())
            .subcommand(leaf_drive_files_get())
            .subcommand(leaf_drive_files_list())
            .subcommand(leaf_drive_files_list_labels())
            .subcommand(leaf_drive_files_modify_labels())
            .subcommand(leaf_drive_files_update())
            .subcommand(leaf_drive_files_watch())
    }

    // drive.operations.get
    fn leaf_drive_operations_get() -> Command {
        Command::new("drive.operations.get")
            .visible_alias("get")
            .about("Gets the latest state of a long-running operation. Clients can use this method to poll the...")
            .long_about("Gets the latest state of a long-running operation. Clients can use this method to poll the operation result at intervals as recommended by the API service.")
            .arg(Arg::new("name").long("name").value_name("NAME").required(true)
                .help("The name of the operation resource."))
            .args(escape_hatch_args())
    }

    // drive.operations
    fn group_drive_operations() -> Command {
        Command::new("operations")
            .about("Methods under drive.operations")
            .subcommand_required(true)
            .subcommand(leaf_drive_operations_get())
    }

    // drive.permissions.create
    fn leaf_drive_permissions_create() -> Command {
        Command::new("drive.permissions.create")
            .visible_alias("create")
            .about("Creates a permission for a file or shared drive. For more information, see Share files, folders...")
            .long_about("Creates a permission for a file or shared drive. For more information, see Share files, folders, and drives. **Warning:** Concurrent permission modifications (such as update or delete) on the same file, folder, or shared driv")
            .arg(Arg::new("email-message").long("email-message").value_name("EMAIL_MESSAGE")
                .help("A plain text custom message to include in the notification email."))
            .arg(Arg::new("enforce-expansive-access").long("enforce-expansive-access").action(ArgAction::SetTrue)
                .help("Deprecated: All requests use the expansive access rules."))
            .arg(Arg::new("enforce-single-parent").long("enforce-single-parent").action(ArgAction::SetTrue)
                .help("Deprecated: See `moveToNewOwnersRoot` for details."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file or shared drive."))
            .arg(Arg::new("move-to-new-owners-root").long("move-to-new-owners-root").action(ArgAction::SetTrue)
                .help("This parameter only takes effect if the item isn't in a shared drive and the request is attempting to transfer the ownership of the item. If set to `true`, the"))
            .arg(Arg::new("send-notification-email").long("send-notification-email").action(ArgAction::SetTrue)
                .help("Whether to send a notification email when sharing to users or groups. This defaults to `true` for users and groups, and is not allowed for other requests. It mu"))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("transfer-ownership").long("transfer-ownership").action(ArgAction::SetTrue)
                .help("Whether to transfer ownership to the specified user and downgrade the current owner to a writer. This parameter is required as an acknowledgement of the side ef"))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator. If set to `true`, and if the following additional conditions are met, the requester is granted access: 1. The file"))
            .args(escape_hatch_args())
    }

    // drive.permissions.delete
    fn leaf_drive_permissions_delete() -> Command {
        Command::new("drive.permissions.delete")
            .visible_alias("delete")
            .about("Deletes a permission. For more information, see Share files, folders, and drives. **Warning:**...")
            .long_about("Deletes a permission. For more information, see Share files, folders, and drives. **Warning:** Concurrent permission modifications (such as update or delete) on the same file, folder, or shared drive aren't supported across a")
            .arg(Arg::new("enforce-expansive-access").long("enforce-expansive-access").action(ArgAction::SetTrue)
                .help("Deprecated: All requests use the expansive access rules."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file or shared drive."))
            .arg(Arg::new("permission-id").long("permission-id").value_name("PERMISSION_ID").required(true)
                .help("The ID of the permission."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator. If set to `true`, and if the following additional conditions are met, the requester is granted access: 1. The file"))
            .args(escape_hatch_args())
    }

    // drive.permissions.get
    fn leaf_drive_permissions_get() -> Command {
        Command::new("drive.permissions.get")
            .visible_alias("get")
            .about("Gets a permission by ID. For more information, see Share files, folders, and drives.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("permission-id").long("permission-id").value_name("PERMISSION_ID").required(true)
                .help("The ID of the permission."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator. If set to `true`, and if the following additional conditions are met, the requester is granted access: 1. The file"))
            .args(escape_hatch_args())
    }

    // drive.permissions.list
    fn leaf_drive_permissions_list() -> Command {
        Command::new("drive.permissions.list")
            .visible_alias("list")
            .about("Lists a file's or shared drive's permissions. For more information, see Share files, folders, and...")
            .long_about("Lists a file's or shared drive's permissions. For more information, see Share files, folders, and drives.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file or shared drive."))
            .arg(Arg::new("include-permissions-for-view").long("include-permissions-for-view").value_name("INCLUDE_PERMISSIONS_FOR_VIEW")
                .help("Specifies which additional view's permissions to include in the response. Only `published` is supported."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of permissions to return. The service may return fewer than this value. If unspecified, at most 100 permissions will be returned for shared d"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from the previous response."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator. If set to `true`, and if the following additional conditions are met, the requester is granted access: 1. The file"))
            .args(escape_hatch_args())
    }

    // drive.permissions.update
    fn leaf_drive_permissions_update() -> Command {
        Command::new("drive.permissions.update")
            .visible_alias("update")
            .about("Updates a permission with patch semantics. For more information, see Share files, folders, and...")
            .long_about("Updates a permission with patch semantics. For more information, see Share files, folders, and drives. **Warning:** Concurrent permission modifications (such as update or delete) on the same file, folder, or shared drive aren")
            .arg(Arg::new("enforce-expansive-access").long("enforce-expansive-access").action(ArgAction::SetTrue)
                .help("Deprecated: All requests use the expansive access rules."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file or shared drive."))
            .arg(Arg::new("permission-id").long("permission-id").value_name("PERMISSION_ID").required(true)
                .help("The ID of the permission."))
            .arg(Arg::new("remove-expiration").long("remove-expiration").action(ArgAction::SetTrue)
                .help("Whether to remove the expiration date."))
            .arg(Arg::new("supports-all-drives").long("supports-all-drives").action(ArgAction::SetTrue)
                .help("Whether the requesting application supports both My Drives and shared drives."))
            .arg(Arg::new("supports-team-drives").long("supports-team-drives").action(ArgAction::SetTrue)
                .help("Deprecated: Use `supportsAllDrives` instead."))
            .arg(Arg::new("transfer-ownership").long("transfer-ownership").action(ArgAction::SetTrue)
                .help("Whether to transfer ownership to the specified user and downgrade the current owner to a writer. This parameter is required as an acknowledgement of the side ef"))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator. If set to `true`, and if the following additional conditions are met, the requester is granted access: 1. The file"))
            .args(escape_hatch_args())
    }

    // drive.permissions
    fn group_drive_permissions() -> Command {
        Command::new("permissions")
            .about("Methods under drive.permissions")
            .subcommand_required(true)
            .subcommand(leaf_drive_permissions_create())
            .subcommand(leaf_drive_permissions_delete())
            .subcommand(leaf_drive_permissions_get())
            .subcommand(leaf_drive_permissions_list())
            .subcommand(leaf_drive_permissions_update())
    }

    // drive.replies.create
    fn leaf_drive_replies_create() -> Command {
        Command::new("drive.replies.create")
            .visible_alias("create")
            .about("Creates a reply to a comment. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .args(escape_hatch_args())
    }

    // drive.replies.delete
    fn leaf_drive_replies_delete() -> Command {
        Command::new("drive.replies.delete")
            .visible_alias("delete")
            .about("Deletes a reply. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("reply-id").long("reply-id").value_name("REPLY_ID").required(true)
                .help("The ID of the reply."))
            .args(escape_hatch_args())
    }

    // drive.replies.get
    fn leaf_drive_replies_get() -> Command {
        Command::new("drive.replies.get")
            .visible_alias("get")
            .about("Gets a reply by ID. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-deleted").long("include-deleted").action(ArgAction::SetTrue)
                .help("Whether to return deleted replies. Deleted replies don't include their original content."))
            .arg(Arg::new("reply-id").long("reply-id").value_name("REPLY_ID").required(true)
                .help("The ID of the reply."))
            .args(escape_hatch_args())
    }

    // drive.replies.list
    fn leaf_drive_replies_list() -> Command {
        Command::new("drive.replies.list")
            .visible_alias("list")
            .about("Lists a comment's replies. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("include-deleted").long("include-deleted").action(ArgAction::SetTrue)
                .help("Whether to include deleted replies. Deleted replies don't include their original content."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of replies to return. The service may return fewer than this value. If unspecified, at most 20 replies will be returned. The maximum value is"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from the previous response."))
            .args(escape_hatch_args())
    }

    // drive.replies.update
    fn leaf_drive_replies_update() -> Command {
        Command::new("drive.replies.update")
            .visible_alias("update")
            .about("Updates a reply with patch semantics. For more information, see Manage comments and replies.")
            .arg(Arg::new("comment-id").long("comment-id").value_name("COMMENT_ID").required(true)
                .help("The ID of the comment."))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("reply-id").long("reply-id").value_name("REPLY_ID").required(true)
                .help("The ID of the reply."))
            .args(escape_hatch_args())
    }

    // drive.replies
    fn group_drive_replies() -> Command {
        Command::new("replies")
            .about("Methods under drive.replies")
            .subcommand_required(true)
            .subcommand(leaf_drive_replies_create())
            .subcommand(leaf_drive_replies_delete())
            .subcommand(leaf_drive_replies_get())
            .subcommand(leaf_drive_replies_list())
            .subcommand(leaf_drive_replies_update())
    }

    // drive.revisions.delete
    fn leaf_drive_revisions_delete() -> Command {
        Command::new("drive.revisions.delete")
            .visible_alias("delete")
            .about("Permanently deletes a file version. You can only delete revisions for files with binary content in...")
            .long_about("Permanently deletes a file version. You can only delete revisions for files with binary content in Google Drive, like images or videos. Revisions for other files, like Google Docs or Sheets, and the last remaining file version can't be deleted. For more information, see [Manage file revisions](https")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("revision-id").long("revision-id").value_name("REVISION_ID").required(true)
                .help("The ID of the revision."))
            .args(escape_hatch_args())
    }

    // drive.revisions.get
    fn leaf_drive_revisions_get() -> Command {
        Command::new("drive.revisions.get")
            .visible_alias("get")
            .about("Gets a revision's metadata or content by ID. For more information, see Manage file revisions.")
            .arg(Arg::new("acknowledge-abuse").long("acknowledge-abuse").action(ArgAction::SetTrue)
                .help("Whether the user is acknowledging the risk of downloading known malware or other abusive files. This is only applicable when the `alt` parameter is set to `medi"))
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("revision-id").long("revision-id").value_name("REVISION_ID").required(true)
                .help("The ID of the revision."))
            .args(escape_hatch_args())
    }

    // drive.revisions.list
    fn leaf_drive_revisions_list() -> Command {
        Command::new("drive.revisions.list")
            .visible_alias("list")
            .about("Lists a file's revisions. For more information, see Manage file revisions. **Important:** The list...")
            .long_about("Lists a file's revisions. For more information, see Manage file revisions. **Important:** The list of revisions returned by this method might be incomplete for files with a large revision history, including frequently edite")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of revisions to return. The service may return fewer than this value. If unspecified, at most 200 revisions will be returned. The maximum val"))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of 'nextPageToken' from the previous response."))
            .args(escape_hatch_args())
    }

    // drive.revisions.update
    fn leaf_drive_revisions_update() -> Command {
        Command::new("drive.revisions.update")
            .visible_alias("update")
            .about("Updates a revision with patch semantics. For more information, see Manage file revisions.")
            .arg(Arg::new("file-id").long("file-id").value_name("FILE_ID").required(true)
                .help("The ID of the file."))
            .arg(Arg::new("revision-id").long("revision-id").value_name("REVISION_ID").required(true)
                .help("The ID of the revision."))
            .args(escape_hatch_args())
    }

    // drive.revisions
    fn group_drive_revisions() -> Command {
        Command::new("revisions")
            .about("Methods under drive.revisions")
            .subcommand_required(true)
            .subcommand(leaf_drive_revisions_delete())
            .subcommand(leaf_drive_revisions_get())
            .subcommand(leaf_drive_revisions_list())
            .subcommand(leaf_drive_revisions_update())
    }

    // drive.teamdrives.create
    fn leaf_drive_teamdrives_create() -> Command {
        Command::new("drive.teamdrives.create")
            .visible_alias("create")
            .about("Deprecated: Use `drives.create` instead.")
            .arg(Arg::new("request-id").long("request-id").value_name("REQUEST_ID").required(true)
                .help("Required. An ID, such as a random UUID, which uniquely identifies this user's request for idempotent creation of a Team Drive. A repeated request by the same us"))
            .args(escape_hatch_args())
    }

    // drive.teamdrives.delete
    fn leaf_drive_teamdrives_delete() -> Command {
        Command::new("drive.teamdrives.delete")
            .visible_alias("delete")
            .about("Deprecated: Use `drives.delete` instead.")
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID").required(true)
                .help("The ID of the Team Drive"))
            .args(escape_hatch_args())
    }

    // drive.teamdrives.get
    fn leaf_drive_teamdrives_get() -> Command {
        Command::new("drive.teamdrives.get")
            .visible_alias("get")
            .about("Deprecated: Use `drives.get` instead.")
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID").required(true)
                .help("The ID of the Team Drive"))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then the requester will be granted access if they are an administrator of the domain to which the T"))
            .args(escape_hatch_args())
    }

    // drive.teamdrives.list
    fn leaf_drive_teamdrives_list() -> Command {
        Command::new("drive.teamdrives.list")
            .visible_alias("list")
            .about("Deprecated: Use `drives.list` instead.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Maximum number of Team Drives to return."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token for Team Drives."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Query string for searching Team Drives."))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then all Team Drives of the domain in which the requester is an administrator are returned."))
            .args(escape_hatch_args())
    }

    // drive.teamdrives.update
    fn leaf_drive_teamdrives_update() -> Command {
        Command::new("drive.teamdrives.update")
            .visible_alias("update")
            .about("Deprecated: Use `drives.update` instead.")
            .arg(Arg::new("team-drive-id").long("team-drive-id").value_name("TEAM_DRIVE_ID").required(true)
                .help("The ID of the Team Drive"))
            .arg(Arg::new("use-domain-admin-access").long("use-domain-admin-access").action(ArgAction::SetTrue)
                .help("Issue the request as a domain administrator; if set to true, then the requester will be granted access if they are an administrator of the domain to which the T"))
            .args(escape_hatch_args())
    }

    // drive.teamdrives
    fn group_drive_teamdrives() -> Command {
        Command::new("teamdrives")
            .about("Methods under drive.teamdrives")
            .subcommand_required(true)
            .subcommand(leaf_drive_teamdrives_create())
            .subcommand(leaf_drive_teamdrives_delete())
            .subcommand(leaf_drive_teamdrives_get())
            .subcommand(leaf_drive_teamdrives_list())
            .subcommand(leaf_drive_teamdrives_update())
    }

    // drive
    fn service_drive() -> Command {
        Command::new("drive")
            .about("Google Drive API operations (v3, 64 methods)")
            .subcommand_required(true)
            .subcommand(group_drive_about())
            .subcommand(group_drive_accessproposals())
            .subcommand(group_drive_approvals())
            .subcommand(group_drive_apps())
            .subcommand(group_drive_changes())
            .subcommand(group_drive_channels())
            .subcommand(group_drive_comments())
            .subcommand(group_drive_drives())
            .subcommand(group_drive_files())
            .subcommand(group_drive_operations())
            .subcommand(group_drive_permissions())
            .subcommand(group_drive_replies())
            .subcommand(group_drive_revisions())
            .subcommand(group_drive_teamdrives())
    }

    // forms.forms.batchUpdate
    fn leaf_forms_forms_batch_update() -> Command {
        Command::new("forms.forms.batchUpdate")
            .visible_alias("batchUpdate")
            .about("Change the form with a batch of updates.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The form ID."))
            .args(escape_hatch_args())
    }

    // forms.forms.create
    fn leaf_forms_forms_create() -> Command {
        Command::new("forms.forms.create")
            .visible_alias("create")
            .about("Create a new form using the title given in the provided form message in the request. *Important:*...")
            .long_about("Create a new form using the title given in the provided form message in the request. *Important:* Only the form.info.title and form.info.document_title fields are copied to the new form. All other fields including the form description, items and settings are disallowed. To create a new form and add")
            .arg(Arg::new("unpublished").long("unpublished").action(ArgAction::SetTrue)
                .help("Optional. Whether the form is unpublished. If set to `true`, the form doesn't accept responses. If set to `false` or unset, the form is published and accepts re"))
            .args(escape_hatch_args())
    }

    // forms.forms.get
    fn leaf_forms_forms_get() -> Command {
        Command::new("forms.forms.get")
            .visible_alias("get")
            .about("Get a form.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The form ID."))
            .args(escape_hatch_args())
    }

    // forms.forms.responses.get
    fn leaf_forms_forms_responses_get() -> Command {
        Command::new("forms.forms.responses.get")
            .visible_alias("get")
            .about("Get one response from the form.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The form ID."))
            .arg(Arg::new("response-id").long("response-id").value_name("RESPONSE_ID").required(true)
                .help("Required. The response ID within the form."))
            .args(escape_hatch_args())
    }

    // forms.forms.responses.list
    fn leaf_forms_forms_responses_list() -> Command {
        Command::new("forms.forms.responses.list")
            .visible_alias("list")
            .about("List a form's responses.")
            .arg(Arg::new("filter").long("filter").value_name("FILTER")
                .help("Which form responses to return. Currently, the only supported filters are: * timestamp > *N* which means to get all form responses submitted after (but not at)"))
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. ID of the Form whose responses to list."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of responses to return. The service may return fewer than this value. If unspecified or zero, at most 5000 responses are returned."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("A page token returned by a previous list response. If this field is set, the form and the values of the filter must be the same as for the original request."))
            .args(escape_hatch_args())
    }

    // forms.forms.responses
    fn group_forms_forms_responses() -> Command {
        Command::new("responses")
            .about("Methods under forms.forms.responses")
            .subcommand_required(true)
            .subcommand(leaf_forms_forms_responses_get())
            .subcommand(leaf_forms_forms_responses_list())
    }

    // forms.forms.setPublishSettings
    fn leaf_forms_forms_set_publish_settings() -> Command {
        Command::new("forms.forms.setPublishSettings")
            .visible_alias("setPublishSettings")
            .about("Updates the publish settings of a form. Legacy forms aren't supported because they don't have the...")
            .long_about("Updates the publish settings of a form. Legacy forms aren't supported because they don't have the `publish_settings` field.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The ID of the form. You can get the id from Form.form_id field."))
            .args(escape_hatch_args())
    }

    // forms.forms.watches.create
    fn leaf_forms_forms_watches_create() -> Command {
        Command::new("forms.forms.watches.create")
            .visible_alias("create")
            .about("Create a new watch. If a watch ID is provided, it must be unused. For each invoking project, the...")
            .long_about("Create a new watch. If a watch ID is provided, it must be unused. For each invoking project, the per form limit is one watch per Watch.EventType. A watch expires seven days after it is created (see Watch.expire_time).")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. ID of the Form to watch."))
            .args(escape_hatch_args())
    }

    // forms.forms.watches.delete
    fn leaf_forms_forms_watches_delete() -> Command {
        Command::new("forms.forms.watches.delete")
            .visible_alias("delete")
            .about("Delete a watch.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The ID of the Form."))
            .arg(Arg::new("watch-id").long("watch-id").value_name("WATCH_ID").required(true)
                .help("Required. The ID of the Watch to delete."))
            .args(escape_hatch_args())
    }

    // forms.forms.watches.list
    fn leaf_forms_forms_watches_list() -> Command {
        Command::new("forms.forms.watches.list")
            .visible_alias("list")
            .about("Return a list of the watches owned by the invoking project. The maximum number of watches is two...")
            .long_about("Return a list of the watches owned by the invoking project. The maximum number of watches is two: For each invoker, the limit is one for each event type per form.")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. ID of the Form whose watches to list."))
            .args(escape_hatch_args())
    }

    // forms.forms.watches.renew
    fn leaf_forms_forms_watches_renew() -> Command {
        Command::new("forms.forms.watches.renew")
            .visible_alias("renew")
            .about("Renew an existing watch for seven days. The state of the watch after renewal is `ACTIVE`, and the...")
            .long_about("Renew an existing watch for seven days. The state of the watch after renewal is `ACTIVE`, and the `expire_time` is seven days from the renewal. Renewing a watch in an error state (e.g. `SUSPENDED`) succeeds if the error is no longer present, but fail otherwise. After a watch has expired, RenewWatch")
            .arg(Arg::new("form-id").long("form-id").value_name("FORM_ID").required(true)
                .help("Required. The ID of the Form."))
            .arg(Arg::new("watch-id").long("watch-id").value_name("WATCH_ID").required(true)
                .help("Required. The ID of the Watch to renew."))
            .args(escape_hatch_args())
    }

    // forms.forms.watches
    fn group_forms_forms_watches() -> Command {
        Command::new("watches")
            .about("Methods under forms.forms.watches")
            .subcommand_required(true)
            .subcommand(leaf_forms_forms_watches_create())
            .subcommand(leaf_forms_forms_watches_delete())
            .subcommand(leaf_forms_forms_watches_list())
            .subcommand(leaf_forms_forms_watches_renew())
    }

    // forms.forms
    fn group_forms_forms() -> Command {
        Command::new("forms")
            .about("Methods under forms.forms")
            .subcommand_required(true)
            .subcommand(leaf_forms_forms_batch_update())
            .subcommand(leaf_forms_forms_create())
            .subcommand(leaf_forms_forms_get())
            .subcommand(group_forms_forms_responses())
            .subcommand(leaf_forms_forms_set_publish_settings())
            .subcommand(group_forms_forms_watches())
    }

    // forms
    fn service_forms() -> Command {
        Command::new("forms")
            .about("Google Forms API operations (v1, 10 methods)")
            .subcommand_required(true)
            .subcommand(group_forms_forms())
    }

    // gmail.users.drafts.create
    fn leaf_gmail_users_drafts_create() -> Command {
        Command::new("gmail.users.drafts.create")
            .visible_alias("create")
            .about("Creates a draft with the `DRAFT` label. For more information, see Create and send draft emails.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts.delete
    fn leaf_gmail_users_drafts_delete() -> Command {
        Command::new("gmail.users.drafts.delete")
            .visible_alias("delete")
            .about("Immediately and permanently deletes the specified draft. Does not simply trash it. For more...")
            .long_about("Immediately and permanently deletes the specified draft. Does not simply trash it. For more information, see Create and send draft emails.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the draft to delete."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts.get
    fn leaf_gmail_users_drafts_get() -> Command {
        Command::new("gmail.users.drafts.get")
            .visible_alias("get")
            .about("Gets the specified draft. For more information, see Create and send draft emails.")
            .arg(Arg::new("param-format").long("param-format").value_name("PARAM_FORMAT").value_parser(["minimal", "full", "raw", "metadata"])
                .help("Discovery parameter `format`, renamed because --format is reserved here. The format to return the draft in."))
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the draft to retrieve."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts.list
    fn leaf_gmail_users_drafts_list() -> Command {
        Command::new("gmail.users.drafts.list")
            .visible_alias("list")
            .about("Lists the drafts in the user's mailbox. For more information, see Create and send draft emails.")
            .arg(Arg::new("include-spam-trash").long("include-spam-trash").action(ArgAction::SetTrue)
                .help("Include drafts from `SPAM` and `TRASH` in the results."))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of drafts to return. This field defaults to 100. The maximum allowed value for this field is 500."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token to retrieve a specific page of results in the list."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Only return draft messages matching the specified query. Supports the same query format as the Gmail search box. For example, `\"from:someuser@example.com rfc822"))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts.send
    fn leaf_gmail_users_drafts_send() -> Command {
        Command::new("gmail.users.drafts.send")
            .visible_alias("send")
            .about("Sends the specified, existing draft to the recipients in the `To`, `Cc`, and `Bcc` headers. For...")
            .long_about("Sends the specified, existing draft to the recipients in the `To`, `Cc`, and `Bcc` headers. For more information, see Create and send draft emails.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts.update
    fn leaf_gmail_users_drafts_update() -> Command {
        Command::new("gmail.users.drafts.update")
            .visible_alias("update")
            .about("Replaces a draft's content. For more information, see Create and send draft emails.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the draft to update."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.drafts
    fn group_gmail_users_drafts() -> Command {
        Command::new("drafts")
            .about("Methods under gmail.users.drafts")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_drafts_create())
            .subcommand(leaf_gmail_users_drafts_delete())
            .subcommand(leaf_gmail_users_drafts_get())
            .subcommand(leaf_gmail_users_drafts_list())
            .subcommand(leaf_gmail_users_drafts_send())
            .subcommand(leaf_gmail_users_drafts_update())
    }

    // gmail.users.getProfile
    fn leaf_gmail_users_get_profile() -> Command {
        Command::new("gmail.users.getProfile")
            .visible_alias("getProfile")
            .about("Gets the current user's Gmail profile.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.history.list
    fn leaf_gmail_users_history_list() -> Command {
        Command::new("gmail.users.history.list")
            .visible_alias("list")
            .about("Lists the history of all changes to the given mailbox. History results are returned in...")
            .long_about("Lists the history of all changes to the given mailbox. History results are returned in chronological order (increasing `historyId`). For more information, see Synchronize clients with Gmail.")
            .arg(Arg::new("history-types").long("history-types").value_name("HISTORY_TYPES").action(ArgAction::Append)
                .help("History types to be returned by the function"))
            .arg(Arg::new("label-id").long("label-id").value_name("LABEL_ID")
                .help("Only return messages with a label matching the ID."))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of history records to return. This field defaults to 100. The maximum allowed value for this field is 500."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token to retrieve a specific page of results in the list."))
            .arg(Arg::new("start-history-id").long("start-history-id").value_name("START_HISTORY_ID")
                .help("Required. Returns history records after the specified `startHistoryId`. The supplied `startHistoryId` should be obtained from the `historyId` of a message, thre"))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.history
    fn group_gmail_users_history() -> Command {
        Command::new("history")
            .about("Methods under gmail.users.history")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_history_list())
    }

    // gmail.users.labels.create
    fn leaf_gmail_users_labels_create() -> Command {
        Command::new("gmail.users.labels.create")
            .visible_alias("create")
            .about("Creates a label. For more information, see Manage labels.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels.delete
    fn leaf_gmail_users_labels_delete() -> Command {
        Command::new("gmail.users.labels.delete")
            .visible_alias("delete")
            .about("Immediately and permanently deletes the specified label and removes it from any messages and...")
            .long_about("Immediately and permanently deletes the specified label and removes it from any messages and threads that it's applied to. For more information, see Manage labels.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the label to delete."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels.get
    fn leaf_gmail_users_labels_get() -> Command {
        Command::new("gmail.users.labels.get")
            .visible_alias("get")
            .about("Gets the specified label. For more information, see Manage labels.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the label to retrieve."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels.list
    fn leaf_gmail_users_labels_list() -> Command {
        Command::new("gmail.users.labels.list")
            .visible_alias("list")
            .about("Lists all labels in the user's mailbox. For more information, see Manage labels.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels.patch
    fn leaf_gmail_users_labels_patch() -> Command {
        Command::new("gmail.users.labels.patch")
            .visible_alias("patch")
            .about("Patch the specified label. For more information, see Manage labels.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the label to update."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels.update
    fn leaf_gmail_users_labels_update() -> Command {
        Command::new("gmail.users.labels.update")
            .visible_alias("update")
            .about("Updates the specified label. For more information, see Manage labels.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the label to update."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.labels
    fn group_gmail_users_labels() -> Command {
        Command::new("labels")
            .about("Methods under gmail.users.labels")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_labels_create())
            .subcommand(leaf_gmail_users_labels_delete())
            .subcommand(leaf_gmail_users_labels_get())
            .subcommand(leaf_gmail_users_labels_list())
            .subcommand(leaf_gmail_users_labels_patch())
            .subcommand(leaf_gmail_users_labels_update())
    }

    // gmail.users.messages.attachments.get
    fn leaf_gmail_users_messages_attachments_get() -> Command {
        Command::new("gmail.users.messages.attachments.get")
            .visible_alias("get")
            .about("Gets the specified message attachment.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the attachment."))
            .arg(Arg::new("message-id").long("message-id").value_name("MESSAGE_ID").required(true)
                .help("The ID of the message containing the attachment."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.attachments
    fn group_gmail_users_messages_attachments() -> Command {
        Command::new("attachments")
            .about("Methods under gmail.users.messages.attachments")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_messages_attachments_get())
    }

    // gmail.users.messages.batchDelete
    fn leaf_gmail_users_messages_batch_delete() -> Command {
        Command::new("gmail.users.messages.batchDelete")
            .visible_alias("batchDelete")
            .about("Deletes many messages by message ID. Provides no guarantees that messages were not already deleted...")
            .long_about("Deletes many messages by message ID. Provides no guarantees that messages were not already deleted or even existed at all.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.batchModify
    fn leaf_gmail_users_messages_batch_modify() -> Command {
        Command::new("gmail.users.messages.batchModify")
            .visible_alias("batchModify")
            .about("Modifies the labels and the Classification Label values on the specified messages. For...")
            .long_about("Modifies the labels and the Classification Label values on the specified messages. For administrators modifying messages for users in their organization, requests require authorization with a service account that has [domain-wi")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.delete
    fn leaf_gmail_users_messages_delete() -> Command {
        Command::new("gmail.users.messages.delete")
            .visible_alias("delete")
            .about("Immediately and permanently deletes the specified message. This operation cannot be undone. Prefer...")
            .long_about("Immediately and permanently deletes the specified message. This operation cannot be undone. Prefer `messages.trash` instead.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the message to delete."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.get
    fn leaf_gmail_users_messages_get() -> Command {
        Command::new("gmail.users.messages.get")
            .visible_alias("get")
            .about("Gets the specified message.")
            .arg(Arg::new("param-format").long("param-format").value_name("PARAM_FORMAT").value_parser(["minimal", "full", "raw", "metadata"])
                .help("Discovery parameter `format`, renamed because --format is reserved here. The format to return the message in."))
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the message to retrieve. This ID is usually retrieved using `messages.list`. The ID is also contained in the result when a message is inserted (`messa"))
            .arg(Arg::new("metadata-headers").long("metadata-headers").value_name("METADATA_HEADERS").action(ArgAction::Append)
                .help("When given and format is `METADATA`, only include headers specified."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.import
    fn leaf_gmail_users_messages_import() -> Command {
        Command::new("gmail.users.messages.import")
            .visible_alias("import")
            .about("Imports a message into only this user's mailbox, with standard email delivery scanning and...")
            .long_about("Imports a message into only this user's mailbox, with standard email delivery scanning and classification similar to receiving via SMTP. This method doesn't perform SPF checks, so it might not work for some spam messages, such as those attempting to perform domain spoofing. This method does not send")
            .arg(Arg::new("deleted").long("deleted").action(ArgAction::SetTrue)
                .help("Mark the email as permanently deleted (not TRASH) and only visible in Google Vault to a Vault administrator. Only used for Google Workspace accounts."))
            .arg(Arg::new("internal-date-source").long("internal-date-source").value_name("INTERNAL_DATE_SOURCE").value_parser(["receivedTime", "dateHeader"])
                .help("Source for Gmail's internal date of the message."))
            .arg(Arg::new("never-mark-spam").long("never-mark-spam").action(ArgAction::SetTrue)
                .help("Ignore the Gmail spam classifier decision and never mark this email as SPAM in the mailbox."))
            .arg(Arg::new("process-for-calendar").long("process-for-calendar").action(ArgAction::SetTrue)
                .help("Process calendar invites in the email and add any extracted meetings to the Google Calendar for this user."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.insert
    fn leaf_gmail_users_messages_insert() -> Command {
        Command::new("gmail.users.messages.insert")
            .visible_alias("insert")
            .about("Directly inserts a message into only this user's mailbox similar to `IMAP APPEND`, bypassing most...")
            .long_about("Directly inserts a message into only this user's mailbox similar to `IMAP APPEND`, bypassing most scanning and classification. Does not send a message. For more information, see Create and send email messages.")
            .arg(Arg::new("deleted").long("deleted").action(ArgAction::SetTrue)
                .help("Mark the email as permanently deleted (not TRASH) and only visible in Google Vault to a Vault administrator. Only used for Google Workspace accounts."))
            .arg(Arg::new("internal-date-source").long("internal-date-source").value_name("INTERNAL_DATE_SOURCE").value_parser(["receivedTime", "dateHeader"])
                .help("Source for Gmail's internal date of the message."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.list
    fn leaf_gmail_users_messages_list() -> Command {
        Command::new("gmail.users.messages.list")
            .visible_alias("list")
            .about("Lists the messages in the user's mailbox. For more information, see List Gmail messages.")
            .arg(Arg::new("include-spam-trash").long("include-spam-trash").action(ArgAction::SetTrue)
                .help("Include messages from `SPAM` and `TRASH` in the results."))
            .arg(Arg::new("label-ids").long("label-ids").value_name("LABEL_IDS").action(ArgAction::Append)
                .help("Only return messages with labels that match all of the specified label IDs. Messages in a thread might have labels that other messages in the same thread don't"))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of messages to return. This field defaults to 100. The maximum allowed value for this field is 500."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token to retrieve a specific page of results in the list."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Only return messages matching the specified query. Supports the same query format as the Gmail search box. For example, `\"from:someuser@example.com rfc822msgid:"))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.modify
    fn leaf_gmail_users_messages_modify() -> Command {
        Command::new("gmail.users.messages.modify")
            .visible_alias("modify")
            .about("Modifies the labels and the Classification Label values on the specified message. For...")
            .long_about("Modifies the labels and the Classification Label values on the specified message. For administrators modifying message for users in their organization, requests require authorization with a service account that has [domain-wide")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the message to modify."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.send
    fn leaf_gmail_users_messages_send() -> Command {
        Command::new("gmail.users.messages.send")
            .visible_alias("send")
            .about("Sends the specified message to the recipients in the `To`, `Cc`, and `Bcc` headers. For more...")
            .long_about("Sends the specified message to the recipients in the `To`, `Cc`, and `Bcc` headers. For more information, see Create and send email messages.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.trash
    fn leaf_gmail_users_messages_trash() -> Command {
        Command::new("gmail.users.messages.trash")
            .visible_alias("trash")
            .about("Moves the specified message to the trash.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the message to Trash."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages.untrash
    fn leaf_gmail_users_messages_untrash() -> Command {
        Command::new("gmail.users.messages.untrash")
            .visible_alias("untrash")
            .about("Removes the specified message from the trash.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the message to remove from Trash."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.messages
    fn group_gmail_users_messages() -> Command {
        Command::new("messages")
            .about("Methods under gmail.users.messages")
            .subcommand_required(true)
            .subcommand(group_gmail_users_messages_attachments())
            .subcommand(leaf_gmail_users_messages_batch_delete())
            .subcommand(leaf_gmail_users_messages_batch_modify())
            .subcommand(leaf_gmail_users_messages_delete())
            .subcommand(leaf_gmail_users_messages_get())
            .subcommand(leaf_gmail_users_messages_import())
            .subcommand(leaf_gmail_users_messages_insert())
            .subcommand(leaf_gmail_users_messages_list())
            .subcommand(leaf_gmail_users_messages_modify())
            .subcommand(leaf_gmail_users_messages_send())
            .subcommand(leaf_gmail_users_messages_trash())
            .subcommand(leaf_gmail_users_messages_untrash())
    }

    // gmail.users.settings.cse.identities.create
    fn leaf_gmail_users_settings_cse_identities_create() -> Command {
        Command::new("gmail.users.settings.cse.identities.create")
            .visible_alias("create")
            .about("Creates and configures a client-side encryption identity that's authorized to send mail from the...")
            .long_about("Creates and configures a client-side encryption identity that's authorized to send mail from the user account. Google publishes the S/MIME certificate to a shared domain-wide directory so that people within a Google Workspace organization can encrypt and send mail to the identity. For administrators")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.identities.delete
    fn leaf_gmail_users_settings_cse_identities_delete() -> Command {
        Command::new("gmail.users.settings.cse.identities.delete")
            .visible_alias("delete")
            .about("Deletes a client-side encryption identity. The authenticated user can no longer use the identity to...")
            .long_about("Deletes a client-side encryption identity. The authenticated user can no longer use the identity to send encrypted messages. You cannot restore the identity after you delete it. Instead, use the CreateCseIdentity method to create another identity with the same configuration. For administrators manag")
            .arg(Arg::new("cse-email-address").long("cse-email-address").value_name("CSE_EMAIL_ADDRESS").required(true)
                .help("The primary email address associated with the client-side encryption identity configuration that's removed."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.identities.get
    fn leaf_gmail_users_settings_cse_identities_get() -> Command {
        Command::new("gmail.users.settings.cse.identities.get")
            .visible_alias("get")
            .about("Retrieves a client-side encryption identity configuration. For administrators managing identities...")
            .long_about("Retrieves a client-side encryption identity configuration. For administrators managing identities and keypairs for users in their organization, requests require authorization with a service account that has [domain-wide delegat")
            .arg(Arg::new("cse-email-address").long("cse-email-address").value_name("CSE_EMAIL_ADDRESS").required(true)
                .help("The primary email address associated with the client-side encryption identity configuration that's retrieved."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.identities.list
    fn leaf_gmail_users_settings_cse_identities_list() -> Command {
        Command::new("gmail.users.settings.cse.identities.list")
            .visible_alias("list")
            .about("Lists the client-side encrypted identities for an authenticated user. For administrators managing...")
            .long_about("Lists the client-side encrypted identities for an authenticated user. For administrators managing identities and keypairs for users in their organization, requests require authorization with a service account that has [domain-w")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The number of identities to return. If not provided, the page size will default to 20 entries."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Pagination token indicating which page of identities to return. If the token is not supplied, then the API will return the first page of results."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.identities.patch
    fn leaf_gmail_users_settings_cse_identities_patch() -> Command {
        Command::new("gmail.users.settings.cse.identities.patch")
            .visible_alias("patch")
            .about("Associates a different key pair with an existing client-side encryption identity. The updated key...")
            .long_about("Associates a different key pair with an existing client-side encryption identity. The updated key pair must validate against Google's S/MIME certificate profiles. For administrators managing identities and keypairs for users in their organization, reque")
            .arg(Arg::new("email-address").long("email-address").value_name("EMAIL_ADDRESS").required(true)
                .help("The email address of the client-side encryption identity to update."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.identities
    fn group_gmail_users_settings_cse_identities() -> Command {
        Command::new("identities")
            .about("Methods under gmail.users.settings.cse.identities")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_cse_identities_create())
            .subcommand(leaf_gmail_users_settings_cse_identities_delete())
            .subcommand(leaf_gmail_users_settings_cse_identities_get())
            .subcommand(leaf_gmail_users_settings_cse_identities_list())
            .subcommand(leaf_gmail_users_settings_cse_identities_patch())
    }

    // gmail.users.settings.cse.keypairs.create
    fn leaf_gmail_users_settings_cse_keypairs_create() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.create")
            .visible_alias("create")
            .about("Creates and uploads a client-side encryption S/MIME public key certificate chain and private key...")
            .long_about("Creates and uploads a client-side encryption S/MIME public key certificate chain and private key metadata for the authenticated user. For administrators managing identities and keypairs for users in their organization, requests require authorization with a [service account](https://developers.google")
            .arg(Arg::new("chain-validation").long("chain-validation").value_name("CHAIN_VALIDATION").value_parser(["all", "none"])
                .help("The type of certificate chain validation to perform at creation. The request will be rejected if the uploaded chain fails to satisfy the requested validation ch"))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs.disable
    fn leaf_gmail_users_settings_cse_keypairs_disable() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.disable")
            .visible_alias("disable")
            .about("Turns off a client-side encryption key pair. The authenticated user can no longer use the key pair...")
            .long_about("Turns off a client-side encryption key pair. The authenticated user can no longer use the key pair to decrypt incoming CSE message texts or sign outgoing CSE mail. To regain access, use the EnableCseKeyPair to turn on the key pair. After 30 days, you can permanently delete the key pair by using the")
            .arg(Arg::new("key-pair-id").long("key-pair-id").value_name("KEY_PAIR_ID").required(true)
                .help("The identifier of the key pair to turn off."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs.enable
    fn leaf_gmail_users_settings_cse_keypairs_enable() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.enable")
            .visible_alias("enable")
            .about("Turns on a client-side encryption key pair that was turned off. The key pair becomes active again...")
            .long_about("Turns on a client-side encryption key pair that was turned off. The key pair becomes active again for any associated client-side encryption identities. For administrators managing identities and keypairs for users in their organization, requests require authorization with a [service account](https:/")
            .arg(Arg::new("key-pair-id").long("key-pair-id").value_name("KEY_PAIR_ID").required(true)
                .help("The identifier of the key pair to turn on."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs.get
    fn leaf_gmail_users_settings_cse_keypairs_get() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.get")
            .visible_alias("get")
            .about("Retrieves an existing client-side encryption key pair. For administrators managing identities and...")
            .long_about("Retrieves an existing client-side encryption key pair. For administrators managing identities and keypairs for users in their organization, requests require authorization with a service account that has [domain-wide delegation")
            .arg(Arg::new("key-pair-id").long("key-pair-id").value_name("KEY_PAIR_ID").required(true)
                .help("The identifier of the key pair to retrieve."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs.list
    fn leaf_gmail_users_settings_cse_keypairs_list() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.list")
            .visible_alias("list")
            .about("Lists client-side encryption key pairs for an authenticated user. For administrators managing...")
            .long_about("Lists client-side encryption key pairs for an authenticated user. For administrators managing identities and keypairs for users in their organization, requests require authorization with a service account that has [domain-wide")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The number of key pairs to return. If not provided, the page size will default to 20 entries."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Pagination token indicating which page of key pairs to return. If the token is not supplied, then the API will return the first page of results."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs.obliterate
    fn leaf_gmail_users_settings_cse_keypairs_obliterate() -> Command {
        Command::new("gmail.users.settings.cse.keypairs.obliterate")
            .visible_alias("obliterate")
            .about("Deletes a client-side encryption key pair permanently and immediately. You can only permanently...")
            .long_about("Deletes a client-side encryption key pair permanently and immediately. You can only permanently delete key pairs that have been turned off for more than 30 days. To turn off a key pair, use the DisableCseKeyPair method. Gmail can't restore or decrypt any messages that were encrypted by an obliterate")
            .arg(Arg::new("key-pair-id").long("key-pair-id").value_name("KEY_PAIR_ID").required(true)
                .help("The identifier of the key pair to obliterate."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The requester's primary email address. To indicate the authenticated user, you can use the special value `me`."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.cse.keypairs
    fn group_gmail_users_settings_cse_keypairs() -> Command {
        Command::new("keypairs")
            .about("Methods under gmail.users.settings.cse.keypairs")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_cse_keypairs_create())
            .subcommand(leaf_gmail_users_settings_cse_keypairs_disable())
            .subcommand(leaf_gmail_users_settings_cse_keypairs_enable())
            .subcommand(leaf_gmail_users_settings_cse_keypairs_get())
            .subcommand(leaf_gmail_users_settings_cse_keypairs_list())
            .subcommand(leaf_gmail_users_settings_cse_keypairs_obliterate())
    }

    // gmail.users.settings.cse
    fn group_gmail_users_settings_cse() -> Command {
        Command::new("cse")
            .about("Methods under gmail.users.settings.cse")
            .subcommand_required(true)
            .subcommand(group_gmail_users_settings_cse_identities())
            .subcommand(group_gmail_users_settings_cse_keypairs())
    }

    // gmail.users.settings.delegates.create
    fn leaf_gmail_users_settings_delegates_create() -> Command {
        Command::new("gmail.users.settings.delegates.create")
            .visible_alias("create")
            .about("Adds a delegate with its verification status set directly to `accepted`, without sending any...")
            .long_about("Adds a delegate with its verification status set directly to `accepted`, without sending any verification email. The delegate user must be a member of the same Google Workspace organization as the delegator user. For more information, see [Manage delegates](https://developers.google.com/workspace/gm")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.delegates.delete
    fn leaf_gmail_users_settings_delegates_delete() -> Command {
        Command::new("gmail.users.settings.delegates.delete")
            .visible_alias("delete")
            .about("Removes the specified delegate (which can be of any verification status), and revokes any...")
            .long_about("Removes the specified delegate (which can be of any verification status), and revokes any verification that may have been required for using it. For more information, see Manage delegates. A delegate user must be referred")
            .arg(Arg::new("delegate-email").long("delegate-email").value_name("DELEGATE_EMAIL").required(true)
                .help("The email address of the user to be removed as a delegate."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.delegates.get
    fn leaf_gmail_users_settings_delegates_get() -> Command {
        Command::new("gmail.users.settings.delegates.get")
            .visible_alias("get")
            .about("Gets the specified delegate. For more information, see Manage delegates. A delegate user must be...")
            .long_about("Gets the specified delegate. For more information, see Manage delegates. A delegate user must be referred to by their primary email address, and not an email alias. This method is only available to service account clients")
            .arg(Arg::new("delegate-email").long("delegate-email").value_name("DELEGATE_EMAIL").required(true)
                .help("The email address of the user whose delegate relationship is to be retrieved."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.delegates.list
    fn leaf_gmail_users_settings_delegates_list() -> Command {
        Command::new("gmail.users.settings.delegates.list")
            .visible_alias("list")
            .about("Lists the delegates for the specified account. For more information, see Manage delegates. This...")
            .long_about("Lists the delegates for the specified account. For more information, see Manage delegates. This method is only available to service account clients that have been delegated domain-wide authority.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.delegates
    fn group_gmail_users_settings_delegates() -> Command {
        Command::new("delegates")
            .about("Methods under gmail.users.settings.delegates")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_delegates_create())
            .subcommand(leaf_gmail_users_settings_delegates_delete())
            .subcommand(leaf_gmail_users_settings_delegates_get())
            .subcommand(leaf_gmail_users_settings_delegates_list())
    }

    // gmail.users.settings.filters.create
    fn leaf_gmail_users_settings_filters_create() -> Command {
        Command::new("gmail.users.settings.filters.create")
            .visible_alias("create")
            .about("Creates a filter. Note: you can only create a maximum of 1,000 filters. For more information, see...")
            .long_about("Creates a filter. Note: you can only create a maximum of 1,000 filters. For more information, see Manage Gmail filters.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.filters.delete
    fn leaf_gmail_users_settings_filters_delete() -> Command {
        Command::new("gmail.users.settings.filters.delete")
            .visible_alias("delete")
            .about("Immediately and permanently deletes the specified filter. For more information, see Manage Gmail...")
            .long_about("Immediately and permanently deletes the specified filter. For more information, see Manage Gmail filters.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the filter to be deleted."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.filters.get
    fn leaf_gmail_users_settings_filters_get() -> Command {
        Command::new("gmail.users.settings.filters.get")
            .visible_alias("get")
            .about("Gets a filter. For more information, see Manage Gmail filters.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the filter to be fetched."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.filters.list
    fn leaf_gmail_users_settings_filters_list() -> Command {
        Command::new("gmail.users.settings.filters.list")
            .visible_alias("list")
            .about("Lists the message filters of a Gmail user. For more information, see Manage Gmail filters.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.filters
    fn group_gmail_users_settings_filters() -> Command {
        Command::new("filters")
            .about("Methods under gmail.users.settings.filters")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_filters_create())
            .subcommand(leaf_gmail_users_settings_filters_delete())
            .subcommand(leaf_gmail_users_settings_filters_get())
            .subcommand(leaf_gmail_users_settings_filters_list())
    }

    // gmail.users.settings.forwardingAddresses.create
    fn leaf_gmail_users_settings_forwarding_addresses_create() -> Command {
        Command::new("gmail.users.settings.forwardingAddresses.create")
            .visible_alias("create")
            .about("Creates a forwarding address. If ownership verification is required, a message will be sent to the...")
            .long_about("Creates a forwarding address. If ownership verification is required, a message will be sent to the recipient and the resource's verification status will be set to `pending`; otherwise, the resource will be created with verification status set to `accepted`. For more information, see [Manage email fo")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.forwardingAddresses.delete
    fn leaf_gmail_users_settings_forwarding_addresses_delete() -> Command {
        Command::new("gmail.users.settings.forwardingAddresses.delete")
            .visible_alias("delete")
            .about("Deletes the specified forwarding address and revokes any verification that may have been required...")
            .long_about("Deletes the specified forwarding address and revokes any verification that may have been required. For more information, see Manage email forwarding. This method is only available to service account clients that have bee")
            .arg(Arg::new("forwarding-email").long("forwarding-email").value_name("FORWARDING_EMAIL").required(true)
                .help("The forwarding address to be deleted."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.forwardingAddresses.get
    fn leaf_gmail_users_settings_forwarding_addresses_get() -> Command {
        Command::new("gmail.users.settings.forwardingAddresses.get")
            .visible_alias("get")
            .about("Gets the specified forwarding address. For more information, see Manage email forwarding.")
            .arg(Arg::new("forwarding-email").long("forwarding-email").value_name("FORWARDING_EMAIL").required(true)
                .help("The forwarding address to be retrieved."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.forwardingAddresses.list
    fn leaf_gmail_users_settings_forwarding_addresses_list() -> Command {
        Command::new("gmail.users.settings.forwardingAddresses.list")
            .visible_alias("list")
            .about("Lists the forwarding addresses for the specified account. For more information, see Manage email...")
            .long_about("Lists the forwarding addresses for the specified account. For more information, see Manage email forwarding.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.forwardingAddresses
    fn group_gmail_users_settings_forwarding_addresses() -> Command {
        Command::new("forwardingAddresses")
            .about("Methods under gmail.users.settings.forwardingAddresses")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_forwarding_addresses_create())
            .subcommand(leaf_gmail_users_settings_forwarding_addresses_delete())
            .subcommand(leaf_gmail_users_settings_forwarding_addresses_get())
            .subcommand(leaf_gmail_users_settings_forwarding_addresses_list())
    }

    // gmail.users.settings.getAutoForwarding
    fn leaf_gmail_users_settings_get_auto_forwarding() -> Command {
        Command::new("gmail.users.settings.getAutoForwarding")
            .visible_alias("getAutoForwarding")
            .about("Gets the auto-forwarding setting for the specified account. For more information, see Manage email...")
            .long_about("Gets the auto-forwarding setting for the specified account. For more information, see Manage email forwarding.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.getImap
    fn leaf_gmail_users_settings_get_imap() -> Command {
        Command::new("gmail.users.settings.getImap")
            .visible_alias("getImap")
            .about("Gets IMAP settings. For more information, see Configure POP and IMAP settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.getLanguage
    fn leaf_gmail_users_settings_get_language() -> Command {
        Command::new("gmail.users.settings.getLanguage")
            .visible_alias("getLanguage")
            .about("Gets language settings. For more information, see Manage language settings.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.getPop
    fn leaf_gmail_users_settings_get_pop() -> Command {
        Command::new("gmail.users.settings.getPop")
            .visible_alias("getPop")
            .about("Gets POP settings. For more information, see Configure POP and IMAP settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.getVacation
    fn leaf_gmail_users_settings_get_vacation() -> Command {
        Command::new("gmail.users.settings.getVacation")
            .visible_alias("getVacation")
            .about("Gets vacation responder settings. For more information, see Manage vacation settings with the Gmail...")
            .long_about("Gets vacation responder settings. For more information, see Manage vacation settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.create
    fn leaf_gmail_users_settings_send_as_create() -> Command {
        Command::new("gmail.users.settings.sendAs.create")
            .visible_alias("create")
            .about("Creates a custom \"from\" send-as alias. If an SMTP MSA is specified, Gmail will attempt to connect...")
            .long_about("Creates a custom \"from\" send-as alias. If an SMTP MSA is specified, Gmail will attempt to connect to the SMTP service to validate the configuration before creating the alias. If ownership verification is required for the alias, a message will be sent to the email address and the resource's verificat")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.delete
    fn leaf_gmail_users_settings_send_as_delete() -> Command {
        Command::new("gmail.users.settings.sendAs.delete")
            .visible_alias("delete")
            .about("Deletes the specified send-as alias. Revokes any verification that may have been required for using...")
            .long_about("Deletes the specified send-as alias. Revokes any verification that may have been required for using it. For more information, see Manage aliases and signatures with the Gmail API. This method is only available t")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The send-as alias to be deleted."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.get
    fn leaf_gmail_users_settings_send_as_get() -> Command {
        Command::new("gmail.users.settings.sendAs.get")
            .visible_alias("get")
            .about("Gets the specified send-as alias. Fails with an HTTP 404 error if the specified address is not a...")
            .long_about("Gets the specified send-as alias. Fails with an HTTP 404 error if the specified address is not a member of the collection. For more information, see Manage aliases and signatures with the Gmail API.")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The send-as alias to be retrieved."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.list
    fn leaf_gmail_users_settings_send_as_list() -> Command {
        Command::new("gmail.users.settings.sendAs.list")
            .visible_alias("list")
            .about("Lists the send-as aliases for the specified account. The result includes the primary send-as...")
            .long_about("Lists the send-as aliases for the specified account. The result includes the primary send-as address associated with the account as well as any custom \"from\" aliases. For more information, see [Manage aliases and signatures with the Gmail API](https://developers.google.com/workspace/gmail/api/guides")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.patch
    fn leaf_gmail_users_settings_send_as_patch() -> Command {
        Command::new("gmail.users.settings.sendAs.patch")
            .visible_alias("patch")
            .about("Patch the specified send-as alias. For more information, see Manage aliases and signatures with the...")
            .long_about("Patch the specified send-as alias. For more information, see Manage aliases and signatures with the Gmail API.")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The send-as alias to be updated."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo.delete
    fn leaf_gmail_users_settings_send_as_smime_info_delete() -> Command {
        Command::new("gmail.users.settings.sendAs.smimeInfo.delete")
            .visible_alias("delete")
            .about("Deletes the specified S/MIME config for the specified send-as alias. For more information, see...")
            .long_about("Deletes the specified S/MIME config for the specified send-as alias. For more information, see Manage S/MIME certificates with the Gmail API.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The immutable ID for the SmimeInfo."))
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The email address that appears in the \"From:\" header for mail sent using this alias."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo.get
    fn leaf_gmail_users_settings_send_as_smime_info_get() -> Command {
        Command::new("gmail.users.settings.sendAs.smimeInfo.get")
            .visible_alias("get")
            .about("Gets the specified S/MIME config for the specified send-as alias. For more information, see Manage...")
            .long_about("Gets the specified S/MIME config for the specified send-as alias. For more information, see Manage S/MIME certificates with the Gmail API.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The immutable ID for the SmimeInfo."))
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The email address that appears in the \"From:\" header for mail sent using this alias."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo.insert
    fn leaf_gmail_users_settings_send_as_smime_info_insert() -> Command {
        Command::new("gmail.users.settings.sendAs.smimeInfo.insert")
            .visible_alias("insert")
            .about("Insert (upload) the given S/MIME config for the specified send-as alias. Note that `pkcs12` format...")
            .long_about("Insert (upload) the given S/MIME config for the specified send-as alias. Note that `pkcs12` format is required for the key. For more information, see Manage S/MIME certificates with the Gmail API.")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The email address that appears in the \"From:\" header for mail sent using this alias."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo.list
    fn leaf_gmail_users_settings_send_as_smime_info_list() -> Command {
        Command::new("gmail.users.settings.sendAs.smimeInfo.list")
            .visible_alias("list")
            .about("Lists S/MIME configs for the specified send-as alias. For more information, see Manage S/MIME...")
            .long_about("Lists S/MIME configs for the specified send-as alias. For more information, see Manage S/MIME certificates with the Gmail API.")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The email address that appears in the \"From:\" header for mail sent using this alias."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo.setDefault
    fn leaf_gmail_users_settings_send_as_smime_info_set_default() -> Command {
        Command::new("gmail.users.settings.sendAs.smimeInfo.setDefault")
            .visible_alias("setDefault")
            .about("Sets the default S/MIME config for the specified send-as alias. For more information, see Manage...")
            .long_about("Sets the default S/MIME config for the specified send-as alias. For more information, see Manage S/MIME certificates with the Gmail API.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The immutable ID for the SmimeInfo."))
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The email address that appears in the \"From:\" header for mail sent using this alias."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.smimeInfo
    fn group_gmail_users_settings_send_as_smime_info() -> Command {
        Command::new("smimeInfo")
            .about("Methods under gmail.users.settings.sendAs.smimeInfo")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_send_as_smime_info_delete())
            .subcommand(leaf_gmail_users_settings_send_as_smime_info_get())
            .subcommand(leaf_gmail_users_settings_send_as_smime_info_insert())
            .subcommand(leaf_gmail_users_settings_send_as_smime_info_list())
            .subcommand(leaf_gmail_users_settings_send_as_smime_info_set_default())
    }

    // gmail.users.settings.sendAs.update
    fn leaf_gmail_users_settings_send_as_update() -> Command {
        Command::new("gmail.users.settings.sendAs.update")
            .visible_alias("update")
            .about("Updates a send-as alias. If a signature is provided, Gmail will sanitize the HTML before saving it...")
            .long_about("Updates a send-as alias. If a signature is provided, Gmail will sanitize the HTML before saving it with the alias. For more information, see Manage aliases and signatures with the Gmail API. Addresses other than")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The send-as alias to be updated."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs.verify
    fn leaf_gmail_users_settings_send_as_verify() -> Command {
        Command::new("gmail.users.settings.sendAs.verify")
            .visible_alias("verify")
            .about("Sends a verification email to the specified send-as alias address. The verification status must be...")
            .long_about("Sends a verification email to the specified send-as alias address. The verification status must be `pending`. For more information, see Manage aliases and signatures with the Gmail API. This method is only avail")
            .arg(Arg::new("send-as-email").long("send-as-email").value_name("SEND_AS_EMAIL").required(true)
                .help("The send-as alias to be verified."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.sendAs
    fn group_gmail_users_settings_send_as() -> Command {
        Command::new("sendAs")
            .about("Methods under gmail.users.settings.sendAs")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_settings_send_as_create())
            .subcommand(leaf_gmail_users_settings_send_as_delete())
            .subcommand(leaf_gmail_users_settings_send_as_get())
            .subcommand(leaf_gmail_users_settings_send_as_list())
            .subcommand(leaf_gmail_users_settings_send_as_patch())
            .subcommand(group_gmail_users_settings_send_as_smime_info())
            .subcommand(leaf_gmail_users_settings_send_as_update())
            .subcommand(leaf_gmail_users_settings_send_as_verify())
    }

    // gmail.users.settings.updateAutoForwarding
    fn leaf_gmail_users_settings_update_auto_forwarding() -> Command {
        Command::new("gmail.users.settings.updateAutoForwarding")
            .visible_alias("updateAutoForwarding")
            .about("Updates the auto-forwarding setting for the specified account. A verified forwarding address must...")
            .long_about("Updates the auto-forwarding setting for the specified account. A verified forwarding address must be specified when auto-forwarding is enabled. For more information, see Manage email forwarding. This method is only avail")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.updateImap
    fn leaf_gmail_users_settings_update_imap() -> Command {
        Command::new("gmail.users.settings.updateImap")
            .visible_alias("updateImap")
            .about("Updates IMAP settings. For more information, see Configure POP and IMAP settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.updateLanguage
    fn leaf_gmail_users_settings_update_language() -> Command {
        Command::new("gmail.users.settings.updateLanguage")
            .visible_alias("updateLanguage")
            .about("Updates language settings. For more information, see Manage language settings. If successful, the...")
            .long_about("Updates language settings. For more information, see Manage language settings. If successful, the return object contains the `displayLanguage` that was saved for the user, which may differ from the value passed into the re")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.updatePop
    fn leaf_gmail_users_settings_update_pop() -> Command {
        Command::new("gmail.users.settings.updatePop")
            .visible_alias("updatePop")
            .about("Updates POP settings. For more information, see Configure POP and IMAP settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings.updateVacation
    fn leaf_gmail_users_settings_update_vacation() -> Command {
        Command::new("gmail.users.settings.updateVacation")
            .visible_alias("updateVacation")
            .about("Updates vacation responder settings. For more information, see Manage vacation settings with the...")
            .long_about("Updates vacation responder settings. For more information, see Manage vacation settings with the Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("User's email address. The special value \"me\" can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.settings
    fn group_gmail_users_settings() -> Command {
        Command::new("settings")
            .about("Methods under gmail.users.settings")
            .subcommand_required(true)
            .subcommand(group_gmail_users_settings_cse())
            .subcommand(group_gmail_users_settings_delegates())
            .subcommand(group_gmail_users_settings_filters())
            .subcommand(group_gmail_users_settings_forwarding_addresses())
            .subcommand(leaf_gmail_users_settings_get_auto_forwarding())
            .subcommand(leaf_gmail_users_settings_get_imap())
            .subcommand(leaf_gmail_users_settings_get_language())
            .subcommand(leaf_gmail_users_settings_get_pop())
            .subcommand(leaf_gmail_users_settings_get_vacation())
            .subcommand(group_gmail_users_settings_send_as())
            .subcommand(leaf_gmail_users_settings_update_auto_forwarding())
            .subcommand(leaf_gmail_users_settings_update_imap())
            .subcommand(leaf_gmail_users_settings_update_language())
            .subcommand(leaf_gmail_users_settings_update_pop())
            .subcommand(leaf_gmail_users_settings_update_vacation())
    }

    // gmail.users.stop
    fn leaf_gmail_users_stop() -> Command {
        Command::new("gmail.users.stop")
            .visible_alias("stop")
            .about("Turn off push notification delivery for the given user mailbox. For more information, see Configure...")
            .long_about("Turn off push notification delivery for the given user mailbox. For more information, see Configure push notifications in Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.delete
    fn leaf_gmail_users_threads_delete() -> Command {
        Command::new("gmail.users.threads.delete")
            .visible_alias("delete")
            .about("Immediately and permanently deletes the specified thread. Any messages that belong to the thread...")
            .long_about("Immediately and permanently deletes the specified thread. Any messages that belong to the thread are also deleted. This operation cannot be undone. Prefer `threads.trash` instead. For more information, see Manage threads.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("ID of the Thread to delete."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.get
    fn leaf_gmail_users_threads_get() -> Command {
        Command::new("gmail.users.threads.get")
            .visible_alias("get")
            .about("Gets the specified thread. For more information, see Manage threads.")
            .arg(Arg::new("param-format").long("param-format").value_name("PARAM_FORMAT").value_parser(["full", "metadata", "minimal"])
                .help("Discovery parameter `format`, renamed because --format is reserved here. The format to return the messages in."))
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the thread to retrieve."))
            .arg(Arg::new("metadata-headers").long("metadata-headers").value_name("METADATA_HEADERS").action(ArgAction::Append)
                .help("When given and format is METADATA, only include headers specified."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.list
    fn leaf_gmail_users_threads_list() -> Command {
        Command::new("gmail.users.threads.list")
            .visible_alias("list")
            .about("Lists the threads in the user's mailbox. For more information, see Manage threads.")
            .arg(Arg::new("include-spam-trash").long("include-spam-trash").action(ArgAction::SetTrue)
                .help("Include threads from `SPAM` and `TRASH` in the results."))
            .arg(Arg::new("label-ids").long("label-ids").value_name("LABEL_IDS").action(ArgAction::Append)
                .help("Only return threads with labels that match all of the specified label IDs."))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of threads to return. This field defaults to 100. The maximum allowed value for this field is 500."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Page token to retrieve a specific page of results in the list."))
            .arg(Arg::new("q").long("q").value_name("Q")
                .help("Only return threads matching the specified query. Supports the same query format as the Gmail search box. For example, `\"from:someuser@example.com rfc822msgid:"))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.modify
    fn leaf_gmail_users_threads_modify() -> Command {
        Command::new("gmail.users.threads.modify")
            .visible_alias("modify")
            .about("Modifies the labels applied to the thread. This applies to all messages in the thread. For more...")
            .long_about("Modifies the labels applied to the thread. This applies to all messages in the thread. For more information, see Manage threads.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the thread to modify."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.trash
    fn leaf_gmail_users_threads_trash() -> Command {
        Command::new("gmail.users.threads.trash")
            .visible_alias("trash")
            .about("Moves the specified thread to the trash. Any messages that belong to the thread are also moved to...")
            .long_about("Moves the specified thread to the trash. Any messages that belong to the thread are also moved to the trash. For more information, see Manage threads.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the thread to Trash."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads.untrash
    fn leaf_gmail_users_threads_untrash() -> Command {
        Command::new("gmail.users.threads.untrash")
            .visible_alias("untrash")
            .about("Removes the specified thread from the trash. Any messages that belong to the thread are also...")
            .long_about("Removes the specified thread from the trash. Any messages that belong to the thread are also removed from the trash. For more information, see Manage threads.")
            .arg(Arg::new("id").long("id").value_name("ID").required(true)
                .help("The ID of the thread to remove from Trash."))
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users.threads
    fn group_gmail_users_threads() -> Command {
        Command::new("threads")
            .about("Methods under gmail.users.threads")
            .subcommand_required(true)
            .subcommand(leaf_gmail_users_threads_delete())
            .subcommand(leaf_gmail_users_threads_get())
            .subcommand(leaf_gmail_users_threads_list())
            .subcommand(leaf_gmail_users_threads_modify())
            .subcommand(leaf_gmail_users_threads_trash())
            .subcommand(leaf_gmail_users_threads_untrash())
    }

    // gmail.users.watch
    fn leaf_gmail_users_watch() -> Command {
        Command::new("gmail.users.watch")
            .visible_alias("watch")
            .about("Set up or update a push notification watch on the given user mailbox. For more information, see...")
            .long_about("Set up or update a push notification watch on the given user mailbox. For more information, see Configure push notifications in Gmail API.")
            .arg(Arg::new("user-id").long("user-id").value_name("USER_ID").required(true)
                .help("The user's email address. The special value `me` can be used to indicate the authenticated user."))
            .args(escape_hatch_args())
    }

    // gmail.users
    fn group_gmail_users() -> Command {
        Command::new("users")
            .about("Methods under gmail.users")
            .subcommand_required(true)
            .subcommand(group_gmail_users_drafts())
            .subcommand(leaf_gmail_users_get_profile())
            .subcommand(group_gmail_users_history())
            .subcommand(group_gmail_users_labels())
            .subcommand(group_gmail_users_messages())
            .subcommand(group_gmail_users_settings())
            .subcommand(leaf_gmail_users_stop())
            .subcommand(group_gmail_users_threads())
            .subcommand(leaf_gmail_users_watch())
    }

    // gmail
    fn service_gmail() -> Command {
        Command::new("gmail")
            .about("Gmail API operations (v1, 79 methods)")
            .subcommand_required(true)
            .subcommand(group_gmail_users())
    }

    // people.contactGroups.batchGet
    fn leaf_people_contact_groups_batch_get() -> Command {
        Command::new("people.contactGroups.batchGet")
            .visible_alias("batchGet")
            .about("Get a list of contact groups owned by the authenticated user by specifying a list of contact group...")
            .long_about("Get a list of contact groups owned by the authenticated user by specifying a list of contact group resource names.")
            .arg(Arg::new("group-fields").long("group-fields").value_name("GROUP_FIELDS")
                .help("Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to e"))
            .arg(Arg::new("max-members").long("max-members").value_name("MAX_MEMBERS").value_parser(clap::value_parser!(i64))
                .help("Optional. Specifies the maximum number of members to return for each group. Defaults to 0 if not set, which will return zero members."))
            .arg(Arg::new("resource-names").long("resource-names").value_name("RESOURCE_NAMES").action(ArgAction::Append)
                .help("Required. The resource names of the contact groups to get. There is a maximum of 200 resource names."))
            .args(escape_hatch_args())
    }

    // people.contactGroups.create
    fn leaf_people_contact_groups_create() -> Command {
        Command::new("people.contactGroups.create")
            .visible_alias("create")
            .about("Create a new contact group owned by the authenticated user. Created contact group names must be...")
            .long_about("Create a new contact group owned by the authenticated user. Created contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoid increased latenc")
            .args(escape_hatch_args())
    }

    // people.contactGroups.delete
    fn leaf_people_contact_groups_delete() -> Command {
        Command::new("people.contactGroups.delete")
            .visible_alias("delete")
            .about("Delete an existing contact group owned by the authenticated user by specifying a contact group...")
            .long_about("Delete an existing contact group owned by the authenticated user by specifying a contact group resource name. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .arg(Arg::new("delete-contacts").long("delete-contacts").action(ArgAction::SetTrue)
                .help("Optional. Set to true to also delete the contacts in the specified group."))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the contact group to delete."))
            .args(escape_hatch_args())
    }

    // people.contactGroups.get
    fn leaf_people_contact_groups_get() -> Command {
        Command::new("people.contactGroups.get")
            .visible_alias("get")
            .about("Get a specific contact group owned by the authenticated user by specifying a contact group resource...")
            .long_about("Get a specific contact group owned by the authenticated user by specifying a contact group resource name.")
            .arg(Arg::new("group-fields").long("group-fields").value_name("GROUP_FIELDS")
                .help("Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to e"))
            .arg(Arg::new("max-members").long("max-members").value_name("MAX_MEMBERS").value_parser(clap::value_parser!(i64))
                .help("Optional. Specifies the maximum number of members to return. Defaults to 0 if not set, which will return zero members."))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the contact group to get."))
            .args(escape_hatch_args())
    }

    // people.contactGroups.list
    fn leaf_people_contact_groups_list() -> Command {
        Command::new("people.contactGroups.list")
            .visible_alias("list")
            .about("List all contact groups owned by the authenticated user. Members of the contact groups are not...")
            .long_about("List all contact groups owned by the authenticated user. Members of the contact groups are not populated.")
            .arg(Arg::new("group-fields").long("group-fields").value_name("GROUP_FIELDS")
                .help("Optional. A field mask to restrict which fields on the group are returned. Defaults to `metadata`, `groupType`, `memberCount`, and `name` if not set or set to e"))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The maximum number of resources to return. Valid values are between 1 and 1000, inclusive. Defaults to 30 if not set or set to 0."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. The next_page_token value returned from a previous call to ListContactGroups. Requests the next page of reso"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Optional. A sync token, returned by a previous call to `contactgroups.list`. Only resources changed since the sync token was created will be returned."))
            .args(escape_hatch_args())
    }

    // people.contactGroups.members.modify
    fn leaf_people_contact_groups_members_modify() -> Command {
        Command::new("people.contactGroups.members.modify")
            .visible_alias("modify")
            .about("Modify the members of a contact group owned by the authenticated user. The only system contact...")
            .long_about("Modify the members of a contact group owned by the authenticated user. The only system contact groups that can have members added are `contactGroups/myContacts` and `contactGroups/starred`. Other system contact groups are deprecated and can only have contacts removed.")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the contact group to modify."))
            .args(escape_hatch_args())
    }

    // people.contactGroups.members
    fn group_people_contact_groups_members() -> Command {
        Command::new("members")
            .about("Methods under people.contactGroups.members")
            .subcommand_required(true)
            .subcommand(leaf_people_contact_groups_members_modify())
    }

    // people.contactGroups.update
    fn leaf_people_contact_groups_update() -> Command {
        Command::new("people.contactGroups.update")
            .visible_alias("update")
            .about("Update the name of an existing contact group owned by the authenticated user. Updated contact group...")
            .long_about("Update the name of an existing contact group owned by the authenticated user. Updated contact group names must be unique to the users contact groups. Attempting to create a group with a duplicate name will return a HTTP 409 error. Mutate requests for the same user should be sent sequentially to avoi")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("The resource name for the contact group, assigned by the server. An ASCII string, in the form of `contactGroups/{contact_group_id}`."))
            .args(escape_hatch_args())
    }

    // people.contactGroups
    fn group_people_contact_groups() -> Command {
        Command::new("contactGroups")
            .about("Methods under people.contactGroups")
            .subcommand_required(true)
            .subcommand(leaf_people_contact_groups_batch_get())
            .subcommand(leaf_people_contact_groups_create())
            .subcommand(leaf_people_contact_groups_delete())
            .subcommand(leaf_people_contact_groups_get())
            .subcommand(leaf_people_contact_groups_list())
            .subcommand(group_people_contact_groups_members())
            .subcommand(leaf_people_contact_groups_update())
    }

    // people.otherContacts.copyOtherContactToMyContactsGroup
    fn leaf_people_other_contacts_copy_other_contact_to_my_contacts_group() -> Command {
        Command::new("people.otherContacts.copyOtherContactToMyContactsGroup")
            .visible_alias("copyOtherContactToMyContactsGroup")
            .about("Copies an \"Other contact\" to a new contact in the user's \"myContacts\" group Mutate requests for the...")
            .long_about("Copies an \"Other contact\" to a new contact in the user's \"myContacts\" group Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the \"Other contact\" to copy."))
            .args(escape_hatch_args())
    }

    // people.otherContacts.list
    fn leaf_people_other_contacts_list() -> Command {
        Command::new("people.otherContacts.list")
            .visible_alias("list")
            .about("List all \"Other contacts\", that is contacts that are not in a contact group. \"Other contacts\" are...")
            .long_about("List all \"Other contacts\", that is contacts that are not in a contact group. \"Other contacts\" are typically auto created contacts from interactions. Sync tokens expire 7 days after the full sync. A request with an expired sync token will get an error with an [google.rpc.ErrorInfo](https://cloud.goog")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of \"Other contacts\" to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("read-mask").long("read-mask").value_name("READ_MASK")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. What values are va"))
            .arg(Arg::new("request-sync-token").long("request-sync-token").action(ArgAction::SetTrue)
                .help("Optional. Whether the response should return `next_sync_token` on the last page of results. It can be used to get incremental changes since the last request by"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT if not set. Possible values for this field are: * READ_SOURCE_TYPE_CONTACT"))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syn"))
            .args(escape_hatch_args())
    }

    // people.otherContacts.search
    fn leaf_people_other_contacts_search() -> Command {
        Command::new("people.otherContacts.search")
            .visible_alias("search")
            .about("Provides a list of contacts in the authenticated user's other contacts that matches the search...")
            .long_about("Provides a list of contacts in the authenticated user's other contacts that matches the search query. The query matches on a contact's `names`, `emailAddresses`, and `phoneNumbers` fields that are from the OTHER_CONTACT source. **IMPORTANT**: Before searching, clients should send a warmup request wi")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of results to return. Defaults to 10 if field is not set, or set to 0. Values greater than 30 will be capped to 30."))
            .arg(Arg::new("param-query").long("param-query").value_name("PARAM_QUERY")
                .help("Discovery parameter `query`, renamed because --query is reserved here. Required. The plain-text query for the request. The query is used to match prefix phrases of the fields on a person. For example, a person with name \"foo name\""))
            .arg(Arg::new("read-mask").long("read-mask").value_name("READ_MASK")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .args(escape_hatch_args())
    }

    // people.otherContacts
    fn group_people_other_contacts() -> Command {
        Command::new("otherContacts")
            .about("Methods under people.otherContacts")
            .subcommand_required(true)
            .subcommand(leaf_people_other_contacts_copy_other_contact_to_my_contacts_group())
            .subcommand(leaf_people_other_contacts_list())
            .subcommand(leaf_people_other_contacts_search())
    }

    // people.people.batchCreateContacts
    fn leaf_people_people_batch_create_contacts() -> Command {
        Command::new("people.people.batchCreateContacts")
            .visible_alias("batchCreateContacts")
            .about("Create a batch of new contacts and return the PersonResponses for the newly Mutate requests for the...")
            .long_about("Create a batch of new contacts and return the PersonResponses for the newly Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .args(escape_hatch_args())
    }

    // people.people.batchDeleteContacts
    fn leaf_people_people_batch_delete_contacts() -> Command {
        Command::new("people.people.batchDeleteContacts")
            .visible_alias("batchDeleteContacts")
            .about("Delete a batch of contacts. Any non-contact data will not be deleted. Mutate requests for the same...")
            .long_about("Delete a batch of contacts. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .args(escape_hatch_args())
    }

    // people.people.batchUpdateContacts
    fn leaf_people_people_batch_update_contacts() -> Command {
        Command::new("people.people.batchUpdateContacts")
            .visible_alias("batchUpdateContacts")
            .about("Update a batch of contacts and return a map of resource names to PersonResponses for the updated...")
            .long_about("Update a batch of contacts and return a map of resource names to PersonResponses for the updated contacts. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .args(escape_hatch_args())
    }

    // people.people.connections.list
    fn leaf_people_people_connections_list() -> Command {
        Command::new("people.people.connections.list")
            .visible_alias("list")
            .about("Provides a list of the authenticated user's contacts. Sync tokens expire 7 days after the full...")
            .long_about("Provides a list of the authenticated user's contacts. Sync tokens expire 7 days after the full sync. A request with an expired sync token will get an error with an google.rpc.ErrorInfo with reason \"EXPIRED_SYNC_TOKEN\". In the case of such an")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of connections to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .arg(Arg::new("request-mask-include-field").long("request-mask-include-field").value_name("REQUEST_MASK_INCLUDE_FIELD")
                .help("Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.ph"))
            .arg(Arg::new("request-sync-token").long("request-sync-token").action(ArgAction::SetTrue)
                .help("Optional. Whether the response should return `next_sync_token` on the last page of results. It can be used to get incremental changes since the last request by"))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name to return connections for. Only `people/me` is valid."))
            .arg(Arg::new("sort-order").long("sort-order").value_name("SORT_ORDER").value_parser(["LAST_MODIFIED_ASCENDING", "LAST_MODIFIED_DESCENDING", "FIRST_NAME_ASCENDING", "LAST_NAME_ASCENDING"])
                .help("Optional. The order in which the connections should be sorted. Defaults to `LAST_MODIFIED_ASCENDING`."))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syn"))
            .args(escape_hatch_args())
    }

    // people.people.connections
    fn group_people_people_connections() -> Command {
        Command::new("connections")
            .about("Methods under people.people.connections")
            .subcommand_required(true)
            .subcommand(leaf_people_people_connections_list())
    }

    // people.people.createContact
    fn leaf_people_people_create_contact() -> Command {
        Command::new("people.people.createContact")
            .visible_alias("createContact")
            .about("Create a new contact and return the person resource for that contact. The request returns a 400...")
            .long_about("Create a new contact and return the person resource for that contact. The request returns a 400 error if more than one field is specified on a field that is a singleton for contact sources: * biographies * birthdays * genders * names Mutate requests for the same user should be sent sequentially to a")
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Defaults to all fi"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set."))
            .args(escape_hatch_args())
    }

    // people.people.deleteContact
    fn leaf_people_people_delete_contact() -> Command {
        Command::new("people.people.deleteContact")
            .visible_alias("deleteContact")
            .about("Delete a contact person. Any non-contact data will not be deleted. Mutate requests for the same...")
            .long_about("Delete a contact person. Any non-contact data will not be deleted. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the contact to delete."))
            .args(escape_hatch_args())
    }

    // people.people.deleteContactPhoto
    fn leaf_people_people_delete_contact_photo() -> Command {
        Command::new("people.people.deleteContactPhoto")
            .visible_alias("deleteContactPhoto")
            .about("Delete a contact's photo. Mutate requests for the same user should be done sequentially to avoid //...")
            .long_about("Delete a contact's photo. Mutate requests for the same user should be done sequentially to avoid // lock contention.")
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Optional. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Defaults to empty i"))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the contact whose photo will be deleted."))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set."))
            .args(escape_hatch_args())
    }

    // people.people.get
    fn leaf_people_people_get() -> Command {
        Command::new("people.people.get")
            .visible_alias("get")
            .about("Provides information about a person by specifying a resource name. Use `people/me` to indicate the...")
            .long_about("Provides information about a person by specifying a resource name. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.")
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Required. A field mask to restrict which fields on the person are returned. Multiple fields can be specified by separating them with commas. Valid values are: *"))
            .arg(Arg::new("request-mask-include-field").long("request-mask-include-field").value_name("REQUEST_MASK_INCLUDE_FIELD")
                .help("Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.ph"))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. The resource name of the person to provide information about. - To get information about the authenticated user, specify `people/me`. - To get informa"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_PROFILE and READ_SOURCE_TYPE_CONTACT if not set."))
            .args(escape_hatch_args())
    }

    // people.people.getBatchGet
    fn leaf_people_people_get_batch_get() -> Command {
        Command::new("people.people.getBatchGet")
            .visible_alias("getBatchGet")
            .about("Provides information about a list of specific people by specifying a list of requested resource...")
            .long_about("Provides information about a list of specific people by specifying a list of requested resource names. Use `people/me` to indicate the authenticated user. The request returns a 400 error if 'personFields' is not specified.")
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .arg(Arg::new("request-mask-include-field").long("request-mask-include-field").value_name("REQUEST_MASK_INCLUDE_FIELD")
                .help("Required. Comma-separated list of person fields to be included in the response. Each path should start with `person.`: for example, `person.names` or `person.ph"))
            .arg(Arg::new("resource-names").long("resource-names").value_name("RESOURCE_NAMES").action(ArgAction::Append)
                .help("Required. The resource names of the people to provide information about. It's repeatable. The URL query parameter should be resourceNames=<name1>&resourceNames="))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set."))
            .args(escape_hatch_args())
    }

    // people.people.listDirectoryPeople
    fn leaf_people_people_list_directory_people() -> Command {
        Command::new("people.people.listDirectoryPeople")
            .visible_alias("listDirectoryPeople")
            .about("Provides a list of domain profiles and domain contacts in the authenticated user's domain...")
            .long_about("Provides a list of domain profiles and domain contacts in the authenticated user's domain directory. When the `sync_token` is specified, resources deleted since the last sync will be returned as a person with `PersonMetadata.deleted` set to true. When the `page_token` or `sync_token` is specified, a")
            .arg(Arg::new("merge-sources").long("merge-sources").value_name("MERGE_SOURCES").action(ArgAction::Append)
                .help("Optional. Additional data to merge into the directory sources if they are connected through verified join keys such as email addresses or phone numbers."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of people to include in the response. Valid values are between 1 and 1000, inclusive. Defaults to 100 if not set or set to 0."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("read-mask").long("read-mask").value_name("READ_MASK")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .arg(Arg::new("request-sync-token").long("request-sync-token").action(ArgAction::SetTrue)
                .help("Optional. Whether the response should return `next_sync_token`. It can be used to get incremental changes since the last request by setting it on the request `s"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Required. Directory sources to return."))
            .arg(Arg::new("sync-token").long("sync-token").value_name("SYNC_TOKEN")
                .help("Optional. A sync token, received from a previous response `next_sync_token` Provide this to retrieve only the resources changed since the last request. When syn"))
            .args(escape_hatch_args())
    }

    // people.people.searchContacts
    fn leaf_people_people_search_contacts() -> Command {
        Command::new("people.people.searchContacts")
            .visible_alias("searchContacts")
            .about("Provides a list of contacts in the authenticated user's grouped contacts that matches the search...")
            .long_about("Provides a list of contacts in the authenticated user's grouped contacts that matches the search query. The query matches on a contact's `names`, `nickNames`, `emailAddresses`, `phoneNumbers`, and `organizations` fields that are from the CONTACT source. **IMPORTANT**: Before searching, clients shoul")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of results to return. Defaults to 10 if field is not set, or set to 0. Values greater than 30 will be capped to 30."))
            .arg(Arg::new("param-query").long("param-query").value_name("PARAM_QUERY")
                .help("Discovery parameter `query`, renamed because --query is reserved here. Required. The plain-text query for the request. The query is used to match prefix phrases of the fields on a person. For example, a person with name \"foo name\""))
            .arg(Arg::new("read-mask").long("read-mask").value_name("READ_MASK")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT if not set."))
            .args(escape_hatch_args())
    }

    // people.people.searchDirectoryPeople
    fn leaf_people_people_search_directory_people() -> Command {
        Command::new("people.people.searchDirectoryPeople")
            .visible_alias("searchDirectoryPeople")
            .about("Provides a list of domain profiles and domain contacts in the authenticated user's domain directory...")
            .long_about("Provides a list of domain profiles and domain contacts in the authenticated user's domain directory that match the search query.")
            .arg(Arg::new("merge-sources").long("merge-sources").value_name("MERGE_SOURCES").action(ArgAction::Append)
                .help("Optional. Additional data to merge into the directory sources if they are connected through verified join keys such as email addresses or phone numbers."))
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("Optional. The number of people to include in the response. Valid values are between 1 and 500, inclusive. Defaults to 100 if not set or set to 0."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Optional. A page token, received from a previous response `next_page_token`. Provide this to retrieve the subsequent page. When paginating, all other parameters"))
            .arg(Arg::new("param-query").long("param-query").value_name("PARAM_QUERY")
                .help("Discovery parameter `query`, renamed because --query is reserved here. Required. Prefix query that matches fields in the person. Does NOT use the read_mask for determining what fields to match."))
            .arg(Arg::new("read-mask").long("read-mask").value_name("READ_MASK")
                .help("Required. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Valid values are:"))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Required. Directory sources to return."))
            .args(escape_hatch_args())
    }

    // people.people.updateContact
    fn leaf_people_people_update_contact() -> Command {
        Command::new("people.people.updateContact")
            .visible_alias("updateContact")
            .about("Update contact data for an existing contact person. Any non-contact data will not be modified. Any...")
            .long_about("Update contact data for an existing contact person. Any non-contact data will not be modified. Any non-contact data in the person to update will be ignored. All fields specified in the `update_mask` will be replaced. The server returns a 400 error if `person.metadata.sources` is not specified for th")
            .arg(Arg::new("person-fields").long("person-fields").value_name("PERSON_FIELDS")
                .help("Optional. A field mask to restrict which fields on each person are returned. Multiple fields can be specified by separating them with commas. Defaults to all fi"))
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("The resource name for the person, assigned by the server. An ASCII string in the form of `people/{person_id}`."))
            .arg(Arg::new("sources").long("sources").value_name("SOURCES").action(ArgAction::Append)
                .help("Optional. A mask of what source types to return. Defaults to READ_SOURCE_TYPE_CONTACT and READ_SOURCE_TYPE_PROFILE if not set."))
            .arg(Arg::new("update-person-fields").long("update-person-fields").value_name("UPDATE_PERSON_FIELDS")
                .help("Required. A field mask to restrict which fields on the person are updated. Multiple fields can be specified by separating them with commas. All updated fields w"))
            .args(escape_hatch_args())
    }

    // people.people.updateContactPhoto
    fn leaf_people_people_update_contact_photo() -> Command {
        Command::new("people.people.updateContactPhoto")
            .visible_alias("updateContactPhoto")
            .about("Update a contact's photo. Mutate requests for the same user should be sent sequentially to avoid...")
            .long_about("Update a contact's photo. Mutate requests for the same user should be sent sequentially to avoid increased latency and failures.")
            .arg(Arg::new("resource-name").long("resource-name").value_name("RESOURCE_NAME").required(true)
                .help("Required. Person resource name"))
            .args(escape_hatch_args())
    }

    // people.people
    fn group_people_people() -> Command {
        Command::new("people")
            .about("Methods under people.people")
            .subcommand_required(true)
            .subcommand(leaf_people_people_batch_create_contacts())
            .subcommand(leaf_people_people_batch_delete_contacts())
            .subcommand(leaf_people_people_batch_update_contacts())
            .subcommand(group_people_people_connections())
            .subcommand(leaf_people_people_create_contact())
            .subcommand(leaf_people_people_delete_contact())
            .subcommand(leaf_people_people_delete_contact_photo())
            .subcommand(leaf_people_people_get())
            .subcommand(leaf_people_people_get_batch_get())
            .subcommand(leaf_people_people_list_directory_people())
            .subcommand(leaf_people_people_search_contacts())
            .subcommand(leaf_people_people_search_directory_people())
            .subcommand(leaf_people_people_update_contact())
            .subcommand(leaf_people_people_update_contact_photo())
    }

    // people
    fn service_people() -> Command {
        Command::new("people")
            .about("People API operations (v1, 24 methods)")
            .subcommand_required(true)
            .subcommand(group_people_contact_groups())
            .subcommand(group_people_other_contacts())
            .subcommand(group_people_people())
    }

    // script.processes.list
    fn leaf_script_processes_list() -> Command {
        Command::new("script.processes.list")
            .visible_alias("list")
            .about("List information about processes made by or on behalf of a user, such as process type and current...")
            .long_about("List information about processes made by or on behalf of a user, such as process type and current status.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of returned processes per page of results. Defaults to 50."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from a previous response."))
            .arg(Arg::new("user-process-filter-deployment-id").long("user-process-filter-deployment-id").value_name("USER_PROCESS_FILTER_DEPLOYMENT_ID")
                .help("Optional field used to limit returned processes to those originating from projects with a specific deployment ID."))
            .arg(Arg::new("user-process-filter-end-time").long("user-process-filter-end-time").value_name("USER_PROCESS_FILTER_END_TIME")
                .help("Optional field used to limit returned processes to those that completed on or before the given timestamp."))
            .arg(Arg::new("user-process-filter-function-name").long("user-process-filter-function-name").value_name("USER_PROCESS_FILTER_FUNCTION_NAME")
                .help("Optional field used to limit returned processes to those originating from a script function with the given function name."))
            .arg(Arg::new("user-process-filter-project-name").long("user-process-filter-project-name").value_name("USER_PROCESS_FILTER_PROJECT_NAME")
                .help("Optional field used to limit returned processes to those originating from projects with project names containing a specific string."))
            .arg(Arg::new("user-process-filter-script-id").long("user-process-filter-script-id").value_name("USER_PROCESS_FILTER_SCRIPT_ID")
                .help("Optional field used to limit returned processes to those originating from projects with a specific script ID."))
            .arg(Arg::new("user-process-filter-start-time").long("user-process-filter-start-time").value_name("USER_PROCESS_FILTER_START_TIME")
                .help("Optional field used to limit returned processes to those that were started on or after the given timestamp."))
            .arg(Arg::new("user-process-filter-statuses").long("user-process-filter-statuses").value_name("USER_PROCESS_FILTER_STATUSES").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified process statuses."))
            .arg(Arg::new("user-process-filter-types").long("user-process-filter-types").value_name("USER_PROCESS_FILTER_TYPES").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified process types."))
            .arg(Arg::new("user-process-filter-user-access-levels").long("user-process-filter-user-access-levels").value_name("USER_PROCESS_FILTER_USER_ACCESS_LEVELS").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified user access levels."))
            .args(escape_hatch_args())
    }

    // script.processes.listScriptProcesses
    fn leaf_script_processes_list_script_processes() -> Command {
        Command::new("script.processes.listScriptProcesses")
            .visible_alias("listScriptProcesses")
            .about("List information about a script's executed processes, such as process type and current status.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of returned processes per page of results. Defaults to 50."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from a previous response."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID")
                .help("The script ID of the project whose processes are listed."))
            .arg(Arg::new("script-process-filter-deployment-id").long("script-process-filter-deployment-id").value_name("SCRIPT_PROCESS_FILTER_DEPLOYMENT_ID")
                .help("Optional field used to limit returned processes to those originating from projects with a specific deployment ID."))
            .arg(Arg::new("script-process-filter-end-time").long("script-process-filter-end-time").value_name("SCRIPT_PROCESS_FILTER_END_TIME")
                .help("Optional field used to limit returned processes to those that completed on or before the given timestamp."))
            .arg(Arg::new("script-process-filter-function-name").long("script-process-filter-function-name").value_name("SCRIPT_PROCESS_FILTER_FUNCTION_NAME")
                .help("Optional field used to limit returned processes to those originating from a script function with the given function name."))
            .arg(Arg::new("script-process-filter-start-time").long("script-process-filter-start-time").value_name("SCRIPT_PROCESS_FILTER_START_TIME")
                .help("Optional field used to limit returned processes to those that were started on or after the given timestamp."))
            .arg(Arg::new("script-process-filter-statuses").long("script-process-filter-statuses").value_name("SCRIPT_PROCESS_FILTER_STATUSES").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified process statuses."))
            .arg(Arg::new("script-process-filter-types").long("script-process-filter-types").value_name("SCRIPT_PROCESS_FILTER_TYPES").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified process types."))
            .arg(Arg::new("script-process-filter-user-access-levels").long("script-process-filter-user-access-levels").value_name("SCRIPT_PROCESS_FILTER_USER_ACCESS_LEVELS").action(ArgAction::Append)
                .help("Optional field used to limit returned processes to those having one of the specified user access levels."))
            .args(escape_hatch_args())
    }

    // script.processes
    fn group_script_processes() -> Command {
        Command::new("processes")
            .about("Methods under script.processes")
            .subcommand_required(true)
            .subcommand(leaf_script_processes_list())
            .subcommand(leaf_script_processes_list_script_processes())
    }

    // script.projects.create
    fn leaf_script_projects_create() -> Command {
        Command::new("script.projects.create")
            .visible_alias("create")
            .about("Creates a new, empty script project with no script files and a base manifest file.")
            .args(escape_hatch_args())
    }

    // script.projects.deployments.create
    fn leaf_script_projects_deployments_create() -> Command {
        Command::new("script.projects.deployments.create")
            .visible_alias("create")
            .about("Creates a deployment of an Apps Script project.")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.deployments.delete
    fn leaf_script_projects_deployments_delete() -> Command {
        Command::new("script.projects.deployments.delete")
            .visible_alias("delete")
            .about("Deletes a deployment of an Apps Script project.")
            .arg(Arg::new("deployment-id").long("deployment-id").value_name("DEPLOYMENT_ID").required(true)
                .help("The deployment ID to be undeployed."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.deployments.get
    fn leaf_script_projects_deployments_get() -> Command {
        Command::new("script.projects.deployments.get")
            .visible_alias("get")
            .about("Gets a deployment of an Apps Script project.")
            .arg(Arg::new("deployment-id").long("deployment-id").value_name("DEPLOYMENT_ID").required(true)
                .help("The deployment ID."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.deployments.list
    fn leaf_script_projects_deployments_list() -> Command {
        Command::new("script.projects.deployments.list")
            .visible_alias("list")
            .about("Lists the deployments of an Apps Script project.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of deployments on each returned page. Defaults to 50."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from a previous response."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.deployments.update
    fn leaf_script_projects_deployments_update() -> Command {
        Command::new("script.projects.deployments.update")
            .visible_alias("update")
            .about("Updates a deployment of an Apps Script project.")
            .arg(Arg::new("deployment-id").long("deployment-id").value_name("DEPLOYMENT_ID").required(true)
                .help("The deployment ID for this deployment."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.deployments
    fn group_script_projects_deployments() -> Command {
        Command::new("deployments")
            .about("Methods under script.projects.deployments")
            .subcommand_required(true)
            .subcommand(leaf_script_projects_deployments_create())
            .subcommand(leaf_script_projects_deployments_delete())
            .subcommand(leaf_script_projects_deployments_get())
            .subcommand(leaf_script_projects_deployments_list())
            .subcommand(leaf_script_projects_deployments_update())
    }

    // script.projects.get
    fn leaf_script_projects_get() -> Command {
        Command::new("script.projects.get")
            .visible_alias("get")
            .about("Gets a script project's metadata.")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.getContent
    fn leaf_script_projects_get_content() -> Command {
        Command::new("script.projects.getContent")
            .visible_alias("getContent")
            .about("Gets the content of the script project, including the code source and metadata for each script file.")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .arg(Arg::new("version-number").long("version-number").value_name("VERSION_NUMBER").value_parser(clap::value_parser!(i64))
                .help("The version number of the project to retrieve. If not provided, the project's HEAD version is returned."))
            .args(escape_hatch_args())
    }

    // script.projects.getMetrics
    fn leaf_script_projects_get_metrics() -> Command {
        Command::new("script.projects.getMetrics")
            .visible_alias("getMetrics")
            .about("Get metrics data for scripts, such as number of executions and active users.")
            .arg(Arg::new("metrics-filter-deployment-id").long("metrics-filter-deployment-id").value_name("METRICS_FILTER_DEPLOYMENT_ID")
                .help("Optional field indicating a specific deployment to retrieve metrics from."))
            .arg(Arg::new("metrics-granularity").long("metrics-granularity").value_name("METRICS_GRANULARITY").value_parser(["UNSPECIFIED_GRANULARITY", "WEEKLY", "DAILY"])
                .help("Required field indicating what granularity of metrics are returned."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("Required field indicating the script to get metrics for."))
            .args(escape_hatch_args())
    }

    // script.projects.updateContent
    fn leaf_script_projects_update_content() -> Command {
        Command::new("script.projects.updateContent")
            .visible_alias("updateContent")
            .about("Updates the content of the specified script project. This content is stored as the HEAD version...")
            .long_about("Updates the content of the specified script project. This content is stored as the HEAD version, and is used when the script is executed as a trigger, in the script editor, in add-on preview mode, or as a web app or Apps Script API in development mode. This clears all the existing files in the proje")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.versions.create
    fn leaf_script_projects_versions_create() -> Command {
        Command::new("script.projects.versions.create")
            .visible_alias("create")
            .about("Creates a new immutable version using the current code, with a unique version number.")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.versions.get
    fn leaf_script_projects_versions_get() -> Command {
        Command::new("script.projects.versions.get")
            .visible_alias("get")
            .about("Gets a version of a script project.")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .arg(Arg::new("version-number").long("version-number").value_name("VERSION_NUMBER").value_parser(clap::value_parser!(i64)).required(true)
                .help("The version number."))
            .args(escape_hatch_args())
    }

    // script.projects.versions.list
    fn leaf_script_projects_versions_list() -> Command {
        Command::new("script.projects.versions.list")
            .visible_alias("list")
            .about("List the versions of a script project.")
            .arg(Arg::new("page-size").long("page-size").value_name("PAGE_SIZE").value_parser(clap::value_parser!(i64))
                .help("The maximum number of versions on each returned page. Defaults to 50."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("The token for continuing a previous list request on the next page. This should be set to the value of `nextPageToken` from a previous response."))
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script project's Drive ID."))
            .args(escape_hatch_args())
    }

    // script.projects.versions
    fn group_script_projects_versions() -> Command {
        Command::new("versions")
            .about("Methods under script.projects.versions")
            .subcommand_required(true)
            .subcommand(leaf_script_projects_versions_create())
            .subcommand(leaf_script_projects_versions_get())
            .subcommand(leaf_script_projects_versions_list())
    }

    // script.projects
    fn group_script_projects() -> Command {
        Command::new("projects")
            .about("Methods under script.projects")
            .subcommand_required(true)
            .subcommand(leaf_script_projects_create())
            .subcommand(group_script_projects_deployments())
            .subcommand(leaf_script_projects_get())
            .subcommand(leaf_script_projects_get_content())
            .subcommand(leaf_script_projects_get_metrics())
            .subcommand(leaf_script_projects_update_content())
            .subcommand(group_script_projects_versions())
    }

    // script.scripts.run
    fn leaf_script_scripts_run() -> Command {
        Command::new("script.scripts.run")
            .visible_alias("run")
            .about("")
            .arg(Arg::new("script-id").long("script-id").value_name("SCRIPT_ID").required(true)
                .help("The script ID of the script to be executed. Find the script ID on the **Project settings** page under \"IDs.\" As multiple executable APIs can be deployed in new"))
            .args(escape_hatch_args())
    }

    // script.scripts
    fn group_script_scripts() -> Command {
        Command::new("scripts")
            .about("Methods under script.scripts")
            .subcommand_required(true)
            .subcommand(leaf_script_scripts_run())
    }

    // script
    fn service_script() -> Command {
        Command::new("script")
            .about("Apps Script API operations (v1, 16 methods)")
            .subcommand_required(true)
            .subcommand(group_script_processes())
            .subcommand(group_script_projects())
            .subcommand(group_script_scripts())
    }

    // searchconsole.searchanalytics.query
    fn leaf_searchconsole_searchanalytics_query() -> Command {
        Command::new("searchconsole.searchanalytics.query")
            .visible_alias("query")
            .about("Query your data with filters and parameters that you define. Returns zero or more rows grouped by...")
            .long_about("Query your data with filters and parameters that you define. Returns zero or more rows grouped by the row keys that you define. You must define a date range of one or more days. When date is one of the group by values, any days without data are omitted from the result list. If you need to know which")
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The site's URL, including protocol. For example: `http://www.example.com/`."))
            .args(escape_hatch_args())
    }

    // searchconsole.searchanalytics
    fn group_searchconsole_searchanalytics() -> Command {
        Command::new("searchanalytics")
            .about("Methods under searchconsole.searchanalytics")
            .subcommand_required(true)
            .subcommand(leaf_searchconsole_searchanalytics_query())
    }

    // searchconsole.sitemaps.delete
    fn leaf_searchconsole_sitemaps_delete() -> Command {
        Command::new("searchconsole.sitemaps.delete")
            .visible_alias("delete")
            .about("Deletes a sitemap from the Sitemaps report. Does not stop Google from crawling this sitemap or the...")
            .long_about("Deletes a sitemap from the Sitemaps report. Does not stop Google from crawling this sitemap or the URLs that were previously crawled in the deleted sitemap.")
            .arg(Arg::new("feedpath").long("feedpath").value_name("FEEDPATH").required(true)
                .help("The URL of the actual sitemap. For example: `http://www.example.com/sitemap.xml`."))
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The site's URL, including protocol. For example: `http://www.example.com/`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sitemaps.get
    fn leaf_searchconsole_sitemaps_get() -> Command {
        Command::new("searchconsole.sitemaps.get")
            .visible_alias("get")
            .about("Retrieves information about a specific sitemap.")
            .arg(Arg::new("feedpath").long("feedpath").value_name("FEEDPATH").required(true)
                .help("The URL of the actual sitemap. For example: `http://www.example.com/sitemap.xml`."))
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The site's URL, including protocol. For example: `http://www.example.com/`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sitemaps.list
    fn leaf_searchconsole_sitemaps_list() -> Command {
        Command::new("searchconsole.sitemaps.list")
            .visible_alias("list")
            .about("Lists the sitemaps-entries submitted for this site, or included in the sitemap index file (if...")
            .long_about("Lists the sitemaps-entries submitted for this site, or included in the sitemap index file (if `sitemapIndex` is specified in the request).")
            .arg(Arg::new("sitemap-index").long("sitemap-index").value_name("SITEMAP_INDEX")
                .help("A URL of a site's sitemap index. For example: `http://www.example.com/sitemapindex.xml`."))
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The site's URL, including protocol. For example: `http://www.example.com/`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sitemaps.submit
    fn leaf_searchconsole_sitemaps_submit() -> Command {
        Command::new("searchconsole.sitemaps.submit")
            .visible_alias("submit")
            .about("Submits a sitemap for a site.")
            .arg(Arg::new("feedpath").long("feedpath").value_name("FEEDPATH").required(true)
                .help("The URL of the actual sitemap. For example: `http://www.example.com/sitemap.xml`."))
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The site's URL, including protocol. For example: `http://www.example.com/`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sitemaps
    fn group_searchconsole_sitemaps() -> Command {
        Command::new("sitemaps")
            .about("Methods under searchconsole.sitemaps")
            .subcommand_required(true)
            .subcommand(leaf_searchconsole_sitemaps_delete())
            .subcommand(leaf_searchconsole_sitemaps_get())
            .subcommand(leaf_searchconsole_sitemaps_list())
            .subcommand(leaf_searchconsole_sitemaps_submit())
    }

    // searchconsole.sites.add
    fn leaf_searchconsole_sites_add() -> Command {
        Command::new("searchconsole.sites.add")
            .visible_alias("add")
            .about("Adds a site to the set of the user's sites in Search Console.")
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The URL of the site to add."))
            .args(escape_hatch_args())
    }

    // searchconsole.sites.delete
    fn leaf_searchconsole_sites_delete() -> Command {
        Command::new("searchconsole.sites.delete")
            .visible_alias("delete")
            .about("Removes a site from the set of the user's Search Console sites.")
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The URI of the property as defined in Search Console. **Examples:** `http://www.example.com/` or `sc-domain:example.com`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sites.get
    fn leaf_searchconsole_sites_get() -> Command {
        Command::new("searchconsole.sites.get")
            .visible_alias("get")
            .about("Retrieves information about specific site.")
            .arg(Arg::new("site-url").long("site-url").value_name("SITE_URL").required(true)
                .help("The URI of the property as defined in Search Console. **Examples:** `http://www.example.com/` or `sc-domain:example.com`."))
            .args(escape_hatch_args())
    }

    // searchconsole.sites.list
    fn leaf_searchconsole_sites_list() -> Command {
        Command::new("searchconsole.sites.list")
            .visible_alias("list")
            .about("Lists the user's Search Console sites.")
            .args(escape_hatch_args())
    }

    // searchconsole.sites
    fn group_searchconsole_sites() -> Command {
        Command::new("sites")
            .about("Methods under searchconsole.sites")
            .subcommand_required(true)
            .subcommand(leaf_searchconsole_sites_add())
            .subcommand(leaf_searchconsole_sites_delete())
            .subcommand(leaf_searchconsole_sites_get())
            .subcommand(leaf_searchconsole_sites_list())
    }

    // searchconsole.urlInspection.index.inspect
    fn leaf_searchconsole_url_inspection_index_inspect() -> Command {
        Command::new("searchconsole.urlInspection.index.inspect")
            .visible_alias("inspect")
            .about("Index inspection.")
            .args(escape_hatch_args())
    }

    // searchconsole.urlInspection.index
    fn group_searchconsole_url_inspection_index() -> Command {
        Command::new("index")
            .about("Methods under searchconsole.urlInspection.index")
            .subcommand_required(true)
            .subcommand(leaf_searchconsole_url_inspection_index_inspect())
    }

    // searchconsole.urlInspection
    fn group_searchconsole_url_inspection() -> Command {
        Command::new("urlInspection")
            .about("Methods under searchconsole.urlInspection")
            .subcommand_required(true)
            .subcommand(group_searchconsole_url_inspection_index())
    }

    // searchconsole.urlTestingTools.mobileFriendlyTest.run
    fn leaf_searchconsole_url_testing_tools_mobile_friendly_test_run() -> Command {
        Command::new("searchconsole.urlTestingTools.mobileFriendlyTest.run")
            .visible_alias("run")
            .about("Runs Mobile-Friendly Test for a given URL.")
            .args(escape_hatch_args())
    }

    // searchconsole.urlTestingTools.mobileFriendlyTest
    fn group_searchconsole_url_testing_tools_mobile_friendly_test() -> Command {
        Command::new("mobileFriendlyTest")
            .about("Methods under searchconsole.urlTestingTools.mobileFriendlyTest")
            .subcommand_required(true)
            .subcommand(leaf_searchconsole_url_testing_tools_mobile_friendly_test_run())
    }

    // searchconsole.urlTestingTools
    fn group_searchconsole_url_testing_tools() -> Command {
        Command::new("urlTestingTools")
            .about("Methods under searchconsole.urlTestingTools")
            .subcommand_required(true)
            .subcommand(group_searchconsole_url_testing_tools_mobile_friendly_test())
    }

    // searchconsole
    fn service_searchconsole() -> Command {
        Command::new("searchconsole")
            .about("Google Search Console API operations (v1, 11 methods)")
            .subcommand_required(true)
            .subcommand(group_searchconsole_searchanalytics())
            .subcommand(group_searchconsole_sitemaps())
            .subcommand(group_searchconsole_sites())
            .subcommand(group_searchconsole_url_inspection())
            .subcommand(group_searchconsole_url_testing_tools())
    }

    // sheets.spreadsheets.batchUpdate
    fn leaf_sheets_spreadsheets_batch_update() -> Command {
        Command::new("sheets.spreadsheets.batchUpdate")
            .visible_alias("batchUpdate")
            .about("Applies one or more updates to the spreadsheet. Each request is validated before being applied. If...")
            .long_about("Applies one or more updates to the spreadsheet. Each request is validated before being applied. If any request is not valid then the entire request will fail and nothing will be applied. Some requests have replies to give you some information about how they are applied. The replies will mirror the r")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The spreadsheet to apply the updates to."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.create
    fn leaf_sheets_spreadsheets_create() -> Command {
        Command::new("sheets.spreadsheets.create")
            .visible_alias("create")
            .about("Creates a spreadsheet, returning the newly created spreadsheet.")
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.developerMetadata.get
    fn leaf_sheets_spreadsheets_developer_metadata_get() -> Command {
        Command::new("sheets.spreadsheets.developerMetadata.get")
            .visible_alias("get")
            .about("Returns the developer metadata with the specified ID. The caller must specify the spreadsheet ID...")
            .long_about("Returns the developer metadata with the specified ID. The caller must specify the spreadsheet ID and the developer metadata's unique metadataId. For more information, see Read, write, and search metadata.")
            .arg(Arg::new("metadata-id").long("metadata-id").value_name("METADATA_ID").value_parser(clap::value_parser!(i64)).required(true)
                .help("The ID of the developer metadata to retrieve."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to retrieve metadata from."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.developerMetadata.search
    fn leaf_sheets_spreadsheets_developer_metadata_search() -> Command {
        Command::new("sheets.spreadsheets.developerMetadata.search")
            .visible_alias("search")
            .about("Returns all developer metadata matching the specified DataFilter. For more information, see Read...")
            .long_about("Returns all developer metadata matching the specified DataFilter. For more information, see Read, write, and search metadata. If the provided DataFilter represents a DeveloperMetadataLookup object, this will return all DeveloperMe")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to retrieve metadata from."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.developerMetadata
    fn group_sheets_spreadsheets_developer_metadata() -> Command {
        Command::new("developerMetadata")
            .about("Methods under sheets.spreadsheets.developerMetadata")
            .subcommand_required(true)
            .subcommand(leaf_sheets_spreadsheets_developer_metadata_get())
            .subcommand(leaf_sheets_spreadsheets_developer_metadata_search())
    }

    // sheets.spreadsheets.get
    fn leaf_sheets_spreadsheets_get() -> Command {
        Command::new("sheets.spreadsheets.get")
            .visible_alias("get")
            .about("Returns the spreadsheet at the given ID. The caller must specify the spreadsheet ID. By default...")
            .long_about("Returns the spreadsheet at the given ID. The caller must specify the spreadsheet ID. By default, data within grids is not returned. You can include grid data in one of 2 ways: * Specify a field mask listing your desired fields")
            .arg(Arg::new("comments-view-mode").long("comments-view-mode").value_name("COMMENTS_VIEW_MODE").value_parser(["COMMENTS_VIEW_MODE_UNSPECIFIED", "COMMENTS_VIEW_MODE_DEFAULT_FOR_CURRENT_ACCESS", "COMMENTS_VIEW_MODE_OMITTED", "COMMENTS_VIEW_MODE_INCLUDED"])
                .help("The comments view mode to apply to the spreadsheet. This allows viewing the spreadsheet with comments omitted or included. If one is not specified, COMMENTS_VIE"))
            .arg(Arg::new("exclude-tables-in-banded-ranges").long("exclude-tables-in-banded-ranges").action(ArgAction::SetTrue)
                .help("True if tables should be excluded in the banded ranges. False if not set."))
            .arg(Arg::new("include-grid-data").long("include-grid-data").action(ArgAction::SetTrue)
                .help("True if grid data should be returned. This parameter is ignored if a field mask was set in the request."))
            .arg(Arg::new("ranges").long("ranges").value_name("RANGES").action(ArgAction::Append)
                .help("The ranges to retrieve from the spreadsheet."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The spreadsheet to request."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.getByDataFilter
    fn leaf_sheets_spreadsheets_get_by_data_filter() -> Command {
        Command::new("sheets.spreadsheets.getByDataFilter")
            .visible_alias("getByDataFilter")
            .about("Returns the spreadsheet at the given ID. The caller must specify the spreadsheet ID. For more...")
            .long_about("Returns the spreadsheet at the given ID. The caller must specify the spreadsheet ID. For more information, see Read, write, and search metadata. This method differs from GetSpreadsheet in that it allows selecting which subsets of")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The spreadsheet to request."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.sheets.copyTo
    fn leaf_sheets_spreadsheets_sheets_copy_to() -> Command {
        Command::new("sheets.spreadsheets.sheets.copyTo")
            .visible_alias("copyTo")
            .about("Copies a single sheet from a spreadsheet to another spreadsheet. Returns the properties of the...")
            .long_about("Copies a single sheet from a spreadsheet to another spreadsheet. Returns the properties of the newly created sheet.")
            .arg(Arg::new("sheet-id").long("sheet-id").value_name("SHEET_ID").value_parser(clap::value_parser!(i64)).required(true)
                .help("The ID of the sheet to copy."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet containing the sheet to copy."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.sheets
    fn group_sheets_spreadsheets_sheets() -> Command {
        Command::new("sheets")
            .about("Methods under sheets.spreadsheets.sheets")
            .subcommand_required(true)
            .subcommand(leaf_sheets_spreadsheets_sheets_copy_to())
    }

    // sheets.spreadsheets.values.append
    fn leaf_sheets_spreadsheets_values_append() -> Command {
        Command::new("sheets.spreadsheets.values.append")
            .visible_alias("append")
            .about("Appends values to a spreadsheet. The input range is used to search for existing data and find a...")
            .long_about("Appends values to a spreadsheet. The input range is used to search for existing data and find a \"table\" within that range. Values will be appended to the next row of the table, starting with the first column of the table. See the [guide](https://developers.google.com/workspace/sheets/api/guides/valu")
            .arg(Arg::new("include-values-in-response").long("include-values-in-response").action(ArgAction::SetTrue)
                .help("Determines if the update response should include the values of the cells that were appended. By default, responses do not include the updated values."))
            .arg(Arg::new("insert-data-option").long("insert-data-option").value_name("INSERT_DATA_OPTION").value_parser(["OVERWRITE", "INSERT_ROWS"])
                .help("How the input data should be inserted."))
            .arg(Arg::new("range").long("range").value_name("RANGE").required(true)
                .help("The A1 notation of a range to search for a logical table of data. Values are appended"))
            .arg(Arg::new("response-date-time-render-option").long("response-date-time-render-option").value_name("RESPONSE_DATE_TIME_RENDER_OPTION").value_parser(["SERIAL_NUMBER", "FORMATTED_STRING"])
                .help("Determines how dates, times, and durations in the response should be rendered. This is ignored if response_value_render_option is FORMATTED_VALUE. The default d"))
            .arg(Arg::new("response-value-render-option").long("response-value-render-option").value_name("RESPONSE_VALUE_RENDER_OPTION").value_parser(["FORMATTED_VALUE", "UNFORMATTED_VALUE", "FORMULA"])
                .help("Determines how values in the response should be rendered. The default render option is FORMATTED_VALUE."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .arg(Arg::new("value-input-option").long("value-input-option").value_name("VALUE_INPUT_OPTION").value_parser(["INPUT_VALUE_OPTION_UNSPECIFIED", "RAW", "USER_ENTERED"])
                .help("How the input data should be interpreted."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchClear
    fn leaf_sheets_spreadsheets_values_batch_clear() -> Command {
        Command::new("sheets.spreadsheets.values.batchClear")
            .visible_alias("batchClear")
            .about("Clears one or more ranges of values from a spreadsheet. The caller must specify the spreadsheet ID...")
            .long_about("Clears one or more ranges of values from a spreadsheet. The caller must specify the spreadsheet ID and one or more ranges. Only values are cleared -- all other properties of the cell (such as formatting and data validation) are kept.")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchClearByDataFilter
    fn leaf_sheets_spreadsheets_values_batch_clear_by_data_filter() -> Command {
        Command::new("sheets.spreadsheets.values.batchClearByDataFilter")
            .visible_alias("batchClearByDataFilter")
            .about("Clears one or more ranges of values from a spreadsheet. For more information, see Read, write, and...")
            .long_about("Clears one or more ranges of values from a spreadsheet. For more information, see Read, write, and search metadata. The caller must specify the spreadsheet ID and one or more DataFilters. Ranges matching any of the specified data")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchGet
    fn leaf_sheets_spreadsheets_values_batch_get() -> Command {
        Command::new("sheets.spreadsheets.values.batchGet")
            .visible_alias("batchGet")
            .about("Returns one or more ranges of values from a spreadsheet. The caller must specify the spreadsheet ID...")
            .long_about("Returns one or more ranges of values from a spreadsheet. The caller must specify the spreadsheet ID and one or more ranges.")
            .arg(Arg::new("date-time-render-option").long("date-time-render-option").value_name("DATE_TIME_RENDER_OPTION").value_parser(["SERIAL_NUMBER", "FORMATTED_STRING"])
                .help("How dates, times, and durations should be represented in the output. This is ignored if value_render_option is FORMATTED_VALUE. The default dateTime render opti"))
            .arg(Arg::new("major-dimension").long("major-dimension").value_name("MAJOR_DIMENSION").value_parser(["DIMENSION_UNSPECIFIED", "ROWS", "COLUMNS"])
                .help("The major dimension that results should use. For example, if the spreadsheet data is: `A1=1,B1=2,A2=3,B2=4`, then requesting `ranges=[\"A1:B2\"],majorDimension=RO"))
            .arg(Arg::new("ranges").long("ranges").value_name("RANGES").action(ArgAction::Append)
                .help("The A1 notation or R1C1 notation of the range to retrieve values from."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to retrieve data from."))
            .arg(Arg::new("value-render-option").long("value-render-option").value_name("VALUE_RENDER_OPTION").value_parser(["FORMATTED_VALUE", "UNFORMATTED_VALUE", "FORMULA"])
                .help("How values should be represented in the output. The default render option is ValueRenderOption.FORMATTED_VALUE."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchGetByDataFilter
    fn leaf_sheets_spreadsheets_values_batch_get_by_data_filter() -> Command {
        Command::new("sheets.spreadsheets.values.batchGetByDataFilter")
            .visible_alias("batchGetByDataFilter")
            .about("Returns one or more ranges of values that match the specified data filters. For more information...")
            .long_about("Returns one or more ranges of values that match the specified data filters. For more information, see Read, write, and search metadata. The caller must specify the spreadsheet ID and one or more DataFilters. Ranges that match any")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to retrieve data from."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchUpdate
    fn leaf_sheets_spreadsheets_values_batch_update() -> Command {
        Command::new("sheets.spreadsheets.values.batchUpdate")
            .visible_alias("batchUpdate")
            .about("Sets values in one or more ranges of a spreadsheet. The caller must specify the spreadsheet ID, a...")
            .long_about("Sets values in one or more ranges of a spreadsheet. The caller must specify the spreadsheet ID, a valueInputOption, and one or more ValueRanges.")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.batchUpdateByDataFilter
    fn leaf_sheets_spreadsheets_values_batch_update_by_data_filter() -> Command {
        Command::new("sheets.spreadsheets.values.batchUpdateByDataFilter")
            .visible_alias("batchUpdateByDataFilter")
            .about("Sets values in one or more ranges of a spreadsheet. For more information, see Read, write, and...")
            .long_about("Sets values in one or more ranges of a spreadsheet. For more information, see Read, write, and search metadata. The caller must specify the spreadsheet ID, a valueInputOption, and one or more DataFilterValueRanges.")
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.clear
    fn leaf_sheets_spreadsheets_values_clear() -> Command {
        Command::new("sheets.spreadsheets.values.clear")
            .visible_alias("clear")
            .about("Clears values from a spreadsheet. The caller must specify the spreadsheet ID and range. Only values...")
            .long_about("Clears values from a spreadsheet. The caller must specify the spreadsheet ID and range. Only values are cleared -- all other properties of the cell (such as formatting, data validation, etc..) are kept.")
            .arg(Arg::new("range").long("range").value_name("RANGE").required(true)
                .help("The A1 notation or R1C1 notation of the values to clear."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.get
    fn leaf_sheets_spreadsheets_values_get() -> Command {
        Command::new("sheets.spreadsheets.values.get")
            .visible_alias("get")
            .about("Returns a range of values from a spreadsheet. The caller must specify the spreadsheet ID and a...")
            .long_about("Returns a range of values from a spreadsheet. The caller must specify the spreadsheet ID and a range.")
            .arg(Arg::new("date-time-render-option").long("date-time-render-option").value_name("DATE_TIME_RENDER_OPTION").value_parser(["SERIAL_NUMBER", "FORMATTED_STRING"])
                .help("How dates, times, and durations should be represented in the output. This is ignored if value_render_option is FORMATTED_VALUE. The default dateTime render opti"))
            .arg(Arg::new("major-dimension").long("major-dimension").value_name("MAJOR_DIMENSION").value_parser(["DIMENSION_UNSPECIFIED", "ROWS", "COLUMNS"])
                .help("The major dimension that results should use. For example, if the spreadsheet data in Sheet1 is: `A1=1,B1=2,A2=3,B2=4`, then requesting `range=Sheet1!A1:B2?major"))
            .arg(Arg::new("range").long("range").value_name("RANGE").required(true)
                .help("The A1 notation or R1C1 notation of the range to retrieve values from."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to retrieve data from."))
            .arg(Arg::new("value-render-option").long("value-render-option").value_name("VALUE_RENDER_OPTION").value_parser(["FORMATTED_VALUE", "UNFORMATTED_VALUE", "FORMULA"])
                .help("How values should be represented in the output. The default render option is FORMATTED_VALUE."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values.update
    fn leaf_sheets_spreadsheets_values_update() -> Command {
        Command::new("sheets.spreadsheets.values.update")
            .visible_alias("update")
            .about("Sets values in a range of a spreadsheet. The caller must specify the spreadsheet ID, range, and a...")
            .long_about("Sets values in a range of a spreadsheet. The caller must specify the spreadsheet ID, range, and a valueInputOption.")
            .arg(Arg::new("include-values-in-response").long("include-values-in-response").action(ArgAction::SetTrue)
                .help("Determines if the update response should include the values of the cells that were updated. By default, responses do not include the updated values. If the rang"))
            .arg(Arg::new("range").long("range").value_name("RANGE").required(true)
                .help("The A1 notation of the values to update."))
            .arg(Arg::new("response-date-time-render-option").long("response-date-time-render-option").value_name("RESPONSE_DATE_TIME_RENDER_OPTION").value_parser(["SERIAL_NUMBER", "FORMATTED_STRING"])
                .help("Determines how dates, times, and durations in the response should be rendered. This is ignored if response_value_render_option is FORMATTED_VALUE. The default d"))
            .arg(Arg::new("response-value-render-option").long("response-value-render-option").value_name("RESPONSE_VALUE_RENDER_OPTION").value_parser(["FORMATTED_VALUE", "UNFORMATTED_VALUE", "FORMULA"])
                .help("Determines how values in the response should be rendered. The default render option is FORMATTED_VALUE."))
            .arg(Arg::new("spreadsheet-id").long("spreadsheet-id").value_name("SPREADSHEET_ID").required(true)
                .help("The ID of the spreadsheet to update."))
            .arg(Arg::new("value-input-option").long("value-input-option").value_name("VALUE_INPUT_OPTION").value_parser(["INPUT_VALUE_OPTION_UNSPECIFIED", "RAW", "USER_ENTERED"])
                .help("How the input data should be interpreted."))
            .args(escape_hatch_args())
    }

    // sheets.spreadsheets.values
    fn group_sheets_spreadsheets_values() -> Command {
        Command::new("values")
            .about("Methods under sheets.spreadsheets.values")
            .subcommand_required(true)
            .subcommand(leaf_sheets_spreadsheets_values_append())
            .subcommand(leaf_sheets_spreadsheets_values_batch_clear())
            .subcommand(leaf_sheets_spreadsheets_values_batch_clear_by_data_filter())
            .subcommand(leaf_sheets_spreadsheets_values_batch_get())
            .subcommand(leaf_sheets_spreadsheets_values_batch_get_by_data_filter())
            .subcommand(leaf_sheets_spreadsheets_values_batch_update())
            .subcommand(leaf_sheets_spreadsheets_values_batch_update_by_data_filter())
            .subcommand(leaf_sheets_spreadsheets_values_clear())
            .subcommand(leaf_sheets_spreadsheets_values_get())
            .subcommand(leaf_sheets_spreadsheets_values_update())
    }

    // sheets.spreadsheets
    fn group_sheets_spreadsheets() -> Command {
        Command::new("spreadsheets")
            .about("Methods under sheets.spreadsheets")
            .subcommand_required(true)
            .subcommand(leaf_sheets_spreadsheets_batch_update())
            .subcommand(leaf_sheets_spreadsheets_create())
            .subcommand(group_sheets_spreadsheets_developer_metadata())
            .subcommand(leaf_sheets_spreadsheets_get())
            .subcommand(leaf_sheets_spreadsheets_get_by_data_filter())
            .subcommand(group_sheets_spreadsheets_sheets())
            .subcommand(group_sheets_spreadsheets_values())
    }

    // sheets
    fn service_sheets() -> Command {
        Command::new("sheets")
            .about("Google Sheets API operations (v4, 17 methods)")
            .subcommand_required(true)
            .subcommand(group_sheets_spreadsheets())
    }

    // slides.presentations.batchUpdate
    fn leaf_slides_presentations_batch_update() -> Command {
        Command::new("slides.presentations.batchUpdate")
            .visible_alias("batchUpdate")
            .about("Applies one or more updates to the presentation. Each request is validated before being applied. If...")
            .long_about("Applies one or more updates to the presentation. Each request is validated before being applied. If any request is not valid, then the entire request will fail and nothing will be applied. Some requests have replies to give you some information about how they are applied. Other requests do not need")
            .arg(Arg::new("presentation-id").long("presentation-id").value_name("PRESENTATION_ID").required(true)
                .help("The presentation to apply the updates to."))
            .args(escape_hatch_args())
    }

    // slides.presentations.create
    fn leaf_slides_presentations_create() -> Command {
        Command::new("slides.presentations.create")
            .visible_alias("create")
            .about("Creates a blank presentation using the title given in the request. If a `presentationId` is...")
            .long_about("Creates a blank presentation using the title given in the request. If a `presentationId` is provided, it is used as the ID of the new presentation. Otherwise, a new ID is generated. Other fields in the request, including any provided content, are ignored. Returns the created presentation.")
            .args(escape_hatch_args())
    }

    // slides.presentations.get
    fn leaf_slides_presentations_get() -> Command {
        Command::new("slides.presentations.get")
            .visible_alias("get")
            .about("Gets the latest version of the specified presentation.")
            .arg(Arg::new("comments-view-mode").long("comments-view-mode").value_name("COMMENTS_VIEW_MODE").value_parser(["COMMENTS_VIEW_MODE_UNSPECIFIED", "COMMENTS_VIEW_MODE_DEFAULT_FOR_CURRENT_ACCESS", "COMMENTS_VIEW_MODE_OMITTED", "COMMENTS_VIEW_MODE_INCLUDED"])
                .help("The comments view mode to apply to the presentation. This allows viewing the presentation with comments omitted or included. If one is not specified, COMMENTS_V"))
            .arg(Arg::new("presentation-id").long("presentation-id").value_name("PRESENTATION_ID").required(true)
                .help("The ID of the presentation to retrieve."))
            .args(escape_hatch_args())
    }

    // slides.presentations.pages.get
    fn leaf_slides_presentations_pages_get() -> Command {
        Command::new("slides.presentations.pages.get")
            .visible_alias("get")
            .about("Gets the latest version of the specified page in the presentation.")
            .arg(Arg::new("comments-view-mode").long("comments-view-mode").value_name("COMMENTS_VIEW_MODE").value_parser(["COMMENTS_VIEW_MODE_UNSPECIFIED", "COMMENTS_VIEW_MODE_DEFAULT_FOR_CURRENT_ACCESS", "COMMENTS_VIEW_MODE_OMITTED", "COMMENTS_VIEW_MODE_INCLUDED"])
                .help("The comments view mode to apply to the page. This allows viewing the page with comments omitted or included. If one is not specified, COMMENTS_VIEW_MODE_OMITTED"))
            .arg(Arg::new("page-object-id").long("page-object-id").value_name("PAGE_OBJECT_ID").required(true)
                .help("The object ID of the page to retrieve."))
            .arg(Arg::new("presentation-id").long("presentation-id").value_name("PRESENTATION_ID").required(true)
                .help("The ID of the presentation to retrieve."))
            .args(escape_hatch_args())
    }

    // slides.presentations.pages.getThumbnail
    fn leaf_slides_presentations_pages_get_thumbnail() -> Command {
        Command::new("slides.presentations.pages.getThumbnail")
            .visible_alias("getThumbnail")
            .about("Generates a thumbnail of the latest version of the specified page in the presentation and returns a...")
            .long_about("Generates a thumbnail of the latest version of the specified page in the presentation and returns a URL to the thumbnail image. This request counts as an expensive read request for quota purposes.")
            .arg(Arg::new("page-object-id").long("page-object-id").value_name("PAGE_OBJECT_ID").required(true)
                .help("The object ID of the page whose thumbnail to retrieve."))
            .arg(Arg::new("presentation-id").long("presentation-id").value_name("PRESENTATION_ID").required(true)
                .help("The ID of the presentation to retrieve."))
            .arg(Arg::new("thumbnail-properties-mime-type").long("thumbnail-properties-mime-type").value_name("THUMBNAIL_PROPERTIES_MIME_TYPE").value_parser(["PNG"])
                .help("The optional mime type of the thumbnail image. If you don't specify the mime type, the mime type defaults to PNG."))
            .arg(Arg::new("thumbnail-properties-thumbnail-size").long("thumbnail-properties-thumbnail-size").value_name("THUMBNAIL_PROPERTIES_THUMBNAIL_SIZE").value_parser(["THUMBNAIL_SIZE_UNSPECIFIED", "LARGE", "MEDIUM", "SMALL", "WIDTH2000_PX"])
                .help("The optional thumbnail image size. If you don't specify the size, the server chooses a default size of the image."))
            .args(escape_hatch_args())
    }

    // slides.presentations.pages
    fn group_slides_presentations_pages() -> Command {
        Command::new("pages")
            .about("Methods under slides.presentations.pages")
            .subcommand_required(true)
            .subcommand(leaf_slides_presentations_pages_get())
            .subcommand(leaf_slides_presentations_pages_get_thumbnail())
    }

    // slides.presentations
    fn group_slides_presentations() -> Command {
        Command::new("presentations")
            .about("Methods under slides.presentations")
            .subcommand_required(true)
            .subcommand(leaf_slides_presentations_batch_update())
            .subcommand(leaf_slides_presentations_create())
            .subcommand(leaf_slides_presentations_get())
            .subcommand(group_slides_presentations_pages())
    }

    // slides
    fn service_slides() -> Command {
        Command::new("slides")
            .about("Google Slides API operations (v1, 5 methods)")
            .subcommand_required(true)
            .subcommand(group_slides_presentations())
    }

    // tasks.tasklists.delete
    fn leaf_tasks_tasklists_delete() -> Command {
        Command::new("tasks.tasklists.delete")
            .visible_alias("delete")
            .about("Deletes the authenticated user's specified task list. If the list contains assigned tasks, both the...")
            .long_about("Deletes the authenticated user's specified task list. If the list contains assigned tasks, both the assigned tasks and the original tasks in the assignment surface (Docs, Chat Spaces) are deleted.")
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasklists.get
    fn leaf_tasks_tasklists_get() -> Command {
        Command::new("tasks.tasklists.get")
            .visible_alias("get")
            .about("Returns the authenticated user's specified task list.")
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasklists.insert
    fn leaf_tasks_tasklists_insert() -> Command {
        Command::new("tasks.tasklists.insert")
            .visible_alias("insert")
            .about("Creates a new task list and adds it to the authenticated user's task lists. A user can have up to...")
            .long_about("Creates a new task list and adds it to the authenticated user's task lists. A user can have up to 2000 lists at a time.")
            .args(escape_hatch_args())
    }

    // tasks.tasklists.list
    fn leaf_tasks_tasklists_list() -> Command {
        Command::new("tasks.tasklists.list")
            .visible_alias("list")
            .about("Returns all the authenticated user's task lists. A user can have up to 2000 lists at a time.")
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of task lists returned on one page. Optional. The default is 1000 (max allowed: 1000)."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying the result page to return. Optional."))
            .args(escape_hatch_args())
    }

    // tasks.tasklists.patch
    fn leaf_tasks_tasklists_patch() -> Command {
        Command::new("tasks.tasklists.patch")
            .visible_alias("patch")
            .about("Updates the authenticated user's specified task list. This method supports patch semantics.")
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasklists.update
    fn leaf_tasks_tasklists_update() -> Command {
        Command::new("tasks.tasklists.update")
            .visible_alias("update")
            .about("Updates the authenticated user's specified task list.")
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasklists
    fn group_tasks_tasklists() -> Command {
        Command::new("tasklists")
            .about("Methods under tasks.tasklists")
            .subcommand_required(true)
            .subcommand(leaf_tasks_tasklists_delete())
            .subcommand(leaf_tasks_tasklists_get())
            .subcommand(leaf_tasks_tasklists_insert())
            .subcommand(leaf_tasks_tasklists_list())
            .subcommand(leaf_tasks_tasklists_patch())
            .subcommand(leaf_tasks_tasklists_update())
    }

    // tasks.tasks.clear
    fn leaf_tasks_tasks_clear() -> Command {
        Command::new("tasks.tasks.clear")
            .visible_alias("clear")
            .about("Clears all completed tasks from the specified task list. The affected tasks will be marked as...")
            .long_about("Clears all completed tasks from the specified task list. The affected tasks will be marked as 'hidden' and no longer be returned by default when retrieving all tasks for a task list.")
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.delete
    fn leaf_tasks_tasks_delete() -> Command {
        Command::new("tasks.tasks.delete")
            .visible_alias("delete")
            .about("Deletes the specified task from the task list. If the task is assigned, both the assigned task and...")
            .long_about("Deletes the specified task from the task list. If the task is assigned, both the assigned task and the original task (in Docs, Chat Spaces) are deleted. To delete the assigned task only, navigate to the assignment surface and unassign the task from there.")
            .arg(Arg::new("task").long("task").value_name("TASK").required(true)
                .help("Task identifier."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.get
    fn leaf_tasks_tasks_get() -> Command {
        Command::new("tasks.tasks.get")
            .visible_alias("get")
            .about("Returns the specified task.")
            .arg(Arg::new("task").long("task").value_name("TASK").required(true)
                .help("Task identifier."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.insert
    fn leaf_tasks_tasks_insert() -> Command {
        Command::new("tasks.tasks.insert")
            .visible_alias("insert")
            .about("Creates a new task on the specified task list. Tasks assigned from Docs or Chat Spaces cannot be...")
            .long_about("Creates a new task on the specified task list. Tasks assigned from Docs or Chat Spaces cannot be inserted from Tasks Public API; they can only be created by assigning them from Docs or Chat Spaces. A user can have up to 20,000 non-hidden tasks per list and up to 100,000 tasks in total at a time.")
            .arg(Arg::new("parent").long("parent").value_name("PARENT")
                .help("Parent task identifier. If the task is created at the top level, this parameter is omitted. An assigned task cannot be a parent task, nor can it have a parent."))
            .arg(Arg::new("previous").long("previous").value_name("PREVIOUS")
                .help("Previous sibling task identifier. If the task is created at the first position among its siblings, this parameter is omitted. Optional."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.list
    fn leaf_tasks_tasks_list() -> Command {
        Command::new("tasks.tasks.list")
            .visible_alias("list")
            .about("Returns all tasks in the specified task list. Doesn't return assigned tasks by default (from Docs...")
            .long_about("Returns all tasks in the specified task list. Doesn't return assigned tasks by default (from Docs, Chat Spaces). A user can have up to 20,000 non-hidden tasks per list and up to 100,000 tasks in total at a time.")
            .arg(Arg::new("completed-max").long("completed-max").value_name("COMPLETED_MAX")
                .help("Upper bound for a task's completion date (as a RFC 3339 timestamp) to filter by. Optional. The default is not to filter by completion date."))
            .arg(Arg::new("completed-min").long("completed-min").value_name("COMPLETED_MIN")
                .help("Lower bound for a task's completion date (as a RFC 3339 timestamp) to filter by. Optional. The default is not to filter by completion date."))
            .arg(Arg::new("due-max").long("due-max").value_name("DUE_MAX")
                .help("Upper bound for a task's due date (as a RFC 3339 timestamp) to filter by. Optional. The default is not to filter by due date."))
            .arg(Arg::new("due-min").long("due-min").value_name("DUE_MIN")
                .help("Lower bound for a task's due date (as a RFC 3339 timestamp) to filter by. Optional. The default is not to filter by due date."))
            .arg(Arg::new("max-results").long("max-results").value_name("MAX_RESULTS").value_parser(clap::value_parser!(i64))
                .help("Maximum number of tasks returned on one page. Optional. The default is 20 (max allowed: 100)."))
            .arg(Arg::new("page-token").long("page-token").value_name("PAGE_TOKEN")
                .help("Token specifying the result page to return. Optional."))
            .arg(Arg::new("show-assigned").long("show-assigned").action(ArgAction::SetTrue)
                .help("Optional. Flag indicating whether tasks assigned to the current user are returned in the result. Optional. The default is False."))
            .arg(Arg::new("show-completed").long("show-completed").action(ArgAction::SetTrue)
                .help("Flag indicating whether completed tasks are returned in the result. Note that showHidden must also be True to show tasks completed in first party clients, such"))
            .arg(Arg::new("show-deleted").long("show-deleted").action(ArgAction::SetTrue)
                .help("Flag indicating whether deleted tasks are returned in the result. Optional. The default is False."))
            .arg(Arg::new("show-hidden").long("show-hidden").action(ArgAction::SetTrue)
                .help("Flag indicating whether hidden tasks are returned in the result. Optional. The default is False."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .arg(Arg::new("updated-min").long("updated-min").value_name("UPDATED_MIN")
                .help("Lower bound for a task's last modification time (as a RFC 3339 timestamp) to filter by. Optional. The default is not to filter by last modification time."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.move
    fn leaf_tasks_tasks_move() -> Command {
        Command::new("tasks.tasks.move")
            .visible_alias("move")
            .about("Moves the specified task to another position in the destination task list. If the destination list...")
            .long_about("Moves the specified task to another position in the destination task list. If the destination list is not specified, the task is moved within its current list. This can include putting it as a child task under a new parent and/or move it to a different position among its sibling tasks. A user can ha")
            .arg(Arg::new("destination-tasklist").long("destination-tasklist").value_name("DESTINATION_TASKLIST")
                .help("Optional. Destination task list identifier. If set, the task is moved from tasklist to the destinationTasklist list. Otherwise the task is moved within its curr"))
            .arg(Arg::new("parent").long("parent").value_name("PARENT")
                .help("Optional. New parent task identifier. If the task is moved to the top level, this parameter is omitted. The task set as parent must exist in the task list and c"))
            .arg(Arg::new("previous").long("previous").value_name("PREVIOUS")
                .help("Optional. New previous sibling task identifier. If the task is moved to the first position among its siblings, this parameter is omitted. The task set as previo"))
            .arg(Arg::new("task").long("task").value_name("TASK").required(true)
                .help("Task identifier."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.patch
    fn leaf_tasks_tasks_patch() -> Command {
        Command::new("tasks.tasks.patch")
            .visible_alias("patch")
            .about("Updates the specified task. This method supports patch semantics.")
            .arg(Arg::new("task").long("task").value_name("TASK").required(true)
                .help("Task identifier."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks.update
    fn leaf_tasks_tasks_update() -> Command {
        Command::new("tasks.tasks.update")
            .visible_alias("update")
            .about("Updates the specified task.")
            .arg(Arg::new("task").long("task").value_name("TASK").required(true)
                .help("Task identifier."))
            .arg(Arg::new("tasklist").long("tasklist").value_name("TASKLIST").required(true)
                .help("Task list identifier."))
            .args(escape_hatch_args())
    }

    // tasks.tasks
    fn group_tasks_tasks() -> Command {
        Command::new("tasks")
            .about("Methods under tasks.tasks")
            .subcommand_required(true)
            .subcommand(leaf_tasks_tasks_clear())
            .subcommand(leaf_tasks_tasks_delete())
            .subcommand(leaf_tasks_tasks_get())
            .subcommand(leaf_tasks_tasks_insert())
            .subcommand(leaf_tasks_tasks_list())
            .subcommand(leaf_tasks_tasks_move())
            .subcommand(leaf_tasks_tasks_patch())
            .subcommand(leaf_tasks_tasks_update())
    }

    // tasks
    fn service_tasks() -> Command {
        Command::new("tasks")
            .about("Google Tasks API operations (v1, 14 methods)")
            .subcommand_required(true)
            .subcommand(group_tasks_tasklists())
            .subcommand(group_tasks_tasks())
    }
}
