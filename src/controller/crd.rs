use std::time::Duration;

use anyhow::bail;

#[cfg(all(feature = "experimental", not(feature = "standard")))]
use gateway_api::apis::experimental::{
    gatewayclasses::GatewayClass, gateways::Gateway, grpcroutes::GRPCRoute, httproutes::HTTPRoute,
    referencegrants::ReferenceGrant, tcproutes::TCPRoute, tlsroutes::TLSRoute, udproutes::UDPRoute,
};

#[cfg(all(feature = "standard", not(feature = "experimental")))]
use gateway_api::apis::standard::{
    gatewayclasses::GatewayClass, gateways::Gateway, httproutes::HTTPRoute,
    referencegrants::ReferenceGrant,
};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::PostParams,
    runtime::{conditions, wait::await_condition},
    Api, Client, CustomResourceExt,
};
use tokio::time::timeout;
use tracing::info;

pub async fn register(client: Client) -> anyhow::Result<()> {
    let api = Api::<CustomResourceDefinition>::all(client.clone());
    #[cfg(any(feature = "standard", feature = "experimental"))]
    // 创建 GatewayClass CRD
    check_create_crd::<GatewayClass>(api.clone()).await?;

    #[cfg(any(feature = "standard", feature = "experimental"))]
    // 创建 Gateway CRD
    check_create_crd::<Gateway>(api.clone()).await?;

    #[cfg(any(feature = "standard", feature = "experimental"))]
    // 创建 HTTPRoute CRD
    check_create_crd::<HTTPRoute>(api.clone()).await?;

    #[cfg(feature = "experimental")]
    // 创建 GRPCRoute CRD
    check_create_crd::<GRPCRoute>(api.clone()).await?;

    #[cfg(feature = "experimental")]
    // 创建 TCPRoute CRD
    check_create_crd::<TCPRoute>(api.clone()).await?;

    #[cfg(feature = "experimental")]
    // 创建 UDPRoute CRD
    check_create_crd::<UDPRoute>(api.clone()).await?;

    #[cfg(feature = "experimental")]
    // 创建 TLSRoute CRD
    check_create_crd::<TLSRoute>(api.clone()).await?;

    #[cfg(any(feature = "standard", feature = "experimental"))]
    // 创建 ReferenceGrant CRD
    check_create_crd::<ReferenceGrant>(api).await?;

    Ok(())
}

#[allow(dead_code)]
async fn check_create_crd<T: CustomResourceExt>(
    api: Api<CustomResourceDefinition>,
) -> anyhow::Result<()> {
    let name = T::crd_name();
    match api.get(name).await {
        Ok(_) => Ok(()),
        Err(kube::Error::Api(resp)) if resp.code == 404 => {
            info!("Registering CRD: {name}");
            let mut data = T::crd();
            // https://github.com/kubernetes/enhancements/pull/1111
            data.metadata.annotations = Some(std::collections::BTreeMap::from([(
                "api-approved.kubernetes.io".to_string(),
                "https://github.com/kubernetes/enhancements/pull/1111".to_string(),
            )]));
            api.create(
                &PostParams {
                    dry_run: false,
                    field_manager: None,
                },
                &data,
            )
            .await?;
            // 等待CRD创建完成，最多重试10次，每次等待500毫秒
            let mut retries = 10;
            while retries > 0 {
                match timeout(
                    Duration::from_millis(500),
                    await_condition(api.clone(), name, conditions::is_crd_established()),
                )
                .await
                {
                    Ok(_) => break, // 成功创建，退出循环
                    Err(e) => {
                        info!("等待中......");
                        retries -= 1;
                        if retries == 0 {
                            // TODO: 需要自定义事件
                            bail!("Timed out waiting for CRD {name} to be established after 30 attempts: {e}")
                        }
                    }
                }
            }
            info!("Registry CRD {name} successful");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
