extern crate env_logger;

use super::{MaxMindDBError, Reader};

use std::net::IpAddr;
use std::str::FromStr;

#[allow(clippy::float_cmp)]
#[test]
fn test_decoder() {
    let _ = env_logger::try_init();

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug, Eq, PartialEq)]
    struct MapXType {
        arrayX: Vec<u32>,
        utf8_stringX: String,
    };

    #[allow(non_snake_case)]
    #[derive(Deserialize, Debug, Eq, PartialEq)]
    struct MapType {
        mapX: MapXType,
    };

    #[derive(Deserialize, Debug)]
    struct TestType {
        array: Vec<u32>,
        boolean: bool,
        bytes: Vec<u8>,
        double: f64,
        float: f32,
        int32: i32,
        map: MapType,
        uint16: u16,
        uint32: u32,
        uint64: u64,
        uint128: Vec<u8>,
        utf8_string: String,
    }

    let r = Reader::open_readfile("test-data/test-data/MaxMind-DB-test-decoder.mmdb");
    if let Err(err) = r {
        panic!(format!("error opening mmdb: {:?}", err));
    }
    let r = r.unwrap();
    let ip: IpAddr = FromStr::from_str("1.1.1.0").unwrap();
    let result: TestType = r.lookup(ip).unwrap();

    assert_eq!(result.array, vec![1u32, 2u32, 3u32]);
    assert_eq!(result.boolean, true);
    assert_eq!(result.bytes, vec![0u8, 0u8, 0u8, 42u8]);
    assert_eq!(result.double, 42.123_456);
    assert_eq!(result.float, 1.1);
    assert_eq!(result.int32, -268_435_456);

    assert_eq!(
        result.map,
        MapType {
            mapX: MapXType {
                arrayX: vec![7, 8, 9],
                utf8_stringX: "hello".to_string(),
            },
        }
    );

    assert_eq!(result.uint16, 100);
    assert_eq!(result.uint32, 268_435_456);
    assert_eq!(result.uint64, 1_152_921_504_606_846_976);
    assert_eq!(
        result.uint128,
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );

    assert_eq!(result.utf8_string, "unicode! ☯ - ♫".to_string());
}

#[test]
fn test_broken_database() {
    let _ = env_logger::try_init();

    let r = Reader::open_readfile("test-data/test-data/GeoIP2-City-Test-Broken-Double-Format.mmdb")
        .ok()
        .unwrap();
    let ip: IpAddr = FromStr::from_str("2001:220::").unwrap();

    #[derive(Deserialize, Debug)]
    struct TestType;
    match r.lookup::<TestType>(ip) {
        Err(e) => assert_eq!(
            e,
            MaxMindDBError::InvalidDatabaseError("double of size 2".to_string())
        ),
        Ok(_) => panic!("Error expected"),
    }
}

#[test]
fn test_missing_database() {
    let _ = env_logger::try_init();

    let r = Reader::open_readfile("file-does-not-exist.mmdb");
    match r {
        Ok(_) => panic!("Received Reader when opening non-existent file"),
        Err(MaxMindDBError::IoError(_)) => assert!(true),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_non_database() {
    let _ = env_logger::try_init();

    let r = Reader::open_readfile("README.md");
    match r {
        Ok(_) => panic!("Received Reader when opening a non-MMDB file"),
        Err(e) => assert_eq!(
            e,
            MaxMindDBError::InvalidDatabaseError(
                "Could not find MaxMind DB metadata \
                 in file."
                    .to_string(),
            )
        ),
    }
}

#[test]
fn test_reader() {
    let _ = env_logger::try_init();

    let sizes = [24usize, 28, 32];
    for record_size in sizes.iter() {
        let versions = [4usize, 6];
        for ip_version in versions.iter() {
            let filename = format!(
                "test-data/test-data/MaxMind-DB-test-ipv{}-{}.mmdb",
                ip_version, record_size
            );
            let reader = Reader::open_readfile(filename).ok().unwrap();

            check_metadata(&reader, *ip_version, *record_size);
            check_ip(&reader, *ip_version);
        }
    }
}

/// Create Reader by explicitly reading the entire file into a buffer.
#[test]
fn test_reader_readfile() {
    let _ = env_logger::try_init();

    let sizes = [24usize, 28, 32];
    for record_size in sizes.iter() {
        let versions = [4usize, 6];
        for ip_version in versions.iter() {
            let filename = format!(
                "test-data/test-data/MaxMind-DB-test-ipv{}-{}.mmdb",
                ip_version, record_size
            );
            let reader = Reader::open_readfile(filename).ok().unwrap();

            check_metadata(&reader, *ip_version, *record_size);
            check_ip(&reader, *ip_version);
        }
    }
}

#[test]
#[cfg(feature = "mmap")]
fn test_reader_mmap() {
    let _ = env_logger::try_init();

    let sizes = [24usize, 28, 32];
    for record_size in sizes.iter() {
        let versions = [4usize, 6];
        for ip_version in versions.iter() {
            let filename = format!(
                "test-data/test-data/MaxMind-DB-test-ipv{}-{}.mmdb",
                ip_version, record_size
            );
            let reader = Reader::open_mmap(filename).ok().unwrap();

            check_metadata(&reader, *ip_version, *record_size);
            check_ip(&reader, *ip_version);
        }
    }
}

#[test]
fn test_lookup_city() {
    use super::geoip2::City;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-City-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("89.160.20.112").unwrap();
    let city: City = reader.lookup(ip).unwrap();

    let iso_code = city.country.and_then(|cy| cy.iso_code);

    assert_eq!(iso_code, Some("SE".to_owned()));
}

#[test]
fn test_lookup_country() {
    use super::geoip2::Country;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-Country-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("89.160.20.112").unwrap();
    let country: Country = reader.lookup(ip).unwrap();
    let country: super::geoip2::model::Country = country.country.unwrap();

    assert_eq!(country.iso_code, Some("SE".to_owned()));
    assert_eq!(country.is_in_european_union, Some(true));
}

#[test]
fn test_lookup_connection_type() {
    use super::geoip2::ConnectionType;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-Connection-Type-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("96.1.20.112").unwrap();
    let connection_type: ConnectionType = reader.lookup(ip).unwrap();

    assert_eq!(
        connection_type.connection_type,
        Some("Cable/DSL".to_owned())
    );
}

#[test]
fn test_lookup_annonymous_ip() {
    use super::geoip2::AnonymousIp;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-Anonymous-IP-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("81.2.69.123").unwrap();
    let anonymous_ip: AnonymousIp = reader.lookup(ip).unwrap();

    assert_eq!(anonymous_ip.is_anonymous, Some(true));
    assert_eq!(anonymous_ip.is_public_proxy, Some(true));
    assert_eq!(anonymous_ip.is_anonymous_vpn, Some(true));
    assert_eq!(anonymous_ip.is_hosting_provider, Some(true));
    assert_eq!(anonymous_ip.is_tor_exit_node, Some(true))
}

#[test]
fn test_lookup_density_income() {
    use super::geoip2::DensityIncome;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-DensityIncome-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("5.83.124.123").unwrap();
    let density_income: DensityIncome = reader.lookup(ip).unwrap();

    assert_eq!(density_income.average_income, Some(32323));
    assert_eq!(density_income.population_density, Some(1232))
}

#[test]
fn test_lookup_domain() {
    use super::geoip2::Domain;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-Domain-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("66.92.80.123").unwrap();
    let domain: Domain = reader.lookup(ip).unwrap();

    assert_eq!(domain.domain, Some("speakeasy.net".to_owned()));
}

#[test]
fn test_lookup_isp() {
    use super::geoip2::Isp;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-ISP-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("12.87.118.123").unwrap();
    let isp: Isp = reader.lookup(ip).unwrap();

    assert_eq!(isp.autonomous_system_number, Some(7018));
    assert_eq!(isp.isp, Some("AT&T Services".to_owned()));
    assert_eq!(isp.organization, Some("AT&T Worldnet Services".to_owned()));
}

#[test]
fn test_lookup_asn() {
    use super::geoip2::Asn;
    let _ = env_logger::try_init();

    let filename = "test-data/test-data/GeoIP2-ISP-Test.mmdb";

    let reader = Reader::open_readfile(filename).unwrap();

    let ip: IpAddr = FromStr::from_str("1.128.0.123").unwrap();
    let asn: Asn = reader.lookup(ip).unwrap();

    assert_eq!(asn.autonomous_system_number, Some(1221));
    assert_eq!(
        asn.autonomous_system_organization,
        Some("Telstra Pty Ltd".to_owned())
    );
}

fn check_metadata<T: AsRef<[u8]>>(reader: &Reader<T>, ip_version: usize, record_size: usize) {
    let metadata = &reader.metadata;

    assert_eq!(metadata.binary_format_major_version, 2u16);

    assert_eq!(metadata.binary_format_minor_version, 0u16);
    assert!(metadata.build_epoch >= 1_397_457_605);
    assert_eq!(metadata.database_type, "Test".to_string());

    assert_eq!(
        *metadata.description[&"en".to_string()],
        "Test Database".to_string()
    );
    assert_eq!(
        *metadata.description[&"zh".to_string()],
        "Test Database Chinese".to_string()
    );

    assert_eq!(metadata.ip_version, ip_version as u16);
    assert_eq!(metadata.languages, vec!["en".to_string(), "zh".to_string()]);

    if ip_version == 4 {
        assert_eq!(metadata.node_count, 164)
    } else {
        assert_eq!(metadata.node_count, 416)
    }

    assert_eq!(metadata.record_size, record_size as u16)
}

fn check_ip<T: AsRef<[u8]>>(reader: &Reader<T>, ip_version: usize) {
    let subnets = match ip_version {
        6 => [
            "::1:ffff:ffff",
            "::2:0:0",
            "::2:0:0",
            "::2:0:0",
            "::2:0:0",
            "::2:0:40",
            "::2:0:40",
            "::2:0:40",
            "::2:0:50",
            "::2:0:50",
            "::2:0:50",
            "::2:0:58",
            "::2:0:58",
        ],
        _ => [
            "1.1.1.1", "1.1.1.2", "1.1.1.2", "1.1.1.4", "1.1.1.4", "1.1.1.4", "1.1.1.4", "1.1.1.8",
            "1.1.1.8", "1.1.1.8", "1.1.1.16", "1.1.1.16", "1.1.1.16",
        ],
    };

    #[derive(Deserialize, Debug)]
    struct IpType {
        ip: String,
    }

    for subnet in subnets.iter() {
        let ip: IpAddr = FromStr::from_str(&subnet).unwrap();
        let value: IpType = reader.lookup(ip).unwrap();

        assert_eq!(value.ip, subnet.to_string());
    }

    let no_record = ["1.1.1.33", "255.254.253.123", "89fa::"];

    for &address in no_record.iter() {
        let ip: IpAddr = FromStr::from_str(address).unwrap();
        match reader.lookup::<IpType>(ip) {
            Ok(v) => panic!("received an unexpected value: {:?}", v),
            Err(e) => assert_eq!(
                e,
                MaxMindDBError::AddressNotFoundError("Address not found in database".to_string())
            ),
        }
    }
}
