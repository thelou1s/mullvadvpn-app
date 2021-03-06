#include "stdafx.h"
#include "mullvadguids.h"
#include <algorithm>
#include <iterator>

//static
WfpObjectRegistry MullvadGuids::BuildRegistry()
{
	const auto detailedRegistry = DetailedRegistry();
	using ValueType = decltype(detailedRegistry)::const_reference;

	std::unordered_set<GUID> registry;

	std::transform(detailedRegistry.begin(), detailedRegistry.end(), std::inserter(registry, registry.end()), [](ValueType value)
	{
		return value.second;
	});

	return registry;
}

//static
DetailedWfpObjectRegistry MullvadGuids::BuildDetailedRegistry()
{
	std::multimap<WfpObjectType, GUID> registry;

	registry.insert(std::make_pair(WfpObjectType::Provider, Provider()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerWhitelist()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerBlacklist()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterBlockAll_Inbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_10_8()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_172_16_12()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_192_168_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_169_254_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Multicast()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Ipv6_fe80_10()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLan_Ipv6_Multicast()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_10_8()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_172_16_12()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_192_168_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_169_254_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLanService_Ipv6_fe80_10()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitLoopback_Inbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpV4_Outbound_Request()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpV6_Outbound_Request()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpV4_Inbound_Response()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitDhcpV6_Inbound_Response()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnRelay()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnel_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnel_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Tunnel_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterRestrictDns_Outbound_Tunnel_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnelService_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, FilterPermitVpnTunnelService_Ipv6()));

	return registry;
}

//static
const WfpObjectRegistry &MullvadGuids::Registry()
{
	static auto registry = BuildRegistry();	// TODO: Thread safety.
	return registry;
}

//static
const DetailedWfpObjectRegistry &MullvadGuids::DetailedRegistry()
{
	static auto registry = BuildDetailedRegistry();	// TODO: Thread safety.
	return registry;
}

//static
const GUID &MullvadGuids::Provider()
{
	static const GUID g =
	{
		0x21e1dab8,
		0xb9db,
		0x43c0,
		{ 0xb3, 0x43, 0xeb, 0x93, 0x65, 0xc7, 0xbd, 0xd2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::SublayerWhitelist()
{
	static const GUID g =
	{
		0x11d1a31a,
		0xd7fa,
		0x469b,
		{ 0xbc, 0x21, 0xcc, 0xe9, 0x2e, 0x35, 0xfe, 0x90 }
	};

	return g;
}

//static
const GUID &MullvadGuids::SublayerBlacklist()
{
	static const GUID g =
	{
		0x843b74f0,
		0xb499,
		0x499a,
		{ 0xac, 0xe3, 0xf9, 0xee, 0xa2, 0x4, 0x89, 0xc1 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xa81c5411,
		0xfd0,
		0x43a9,
		{ 0xa9, 0xbe, 0x31, 0x3f, 0x29, 0x9d, 0xe6, 0x4f }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x8ae5c389,
		0xd604,
		0x43df,
		{ 0x87, 0x4a, 0x5c, 0x86, 0x76, 0xc9, 0xc2, 0xb8 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Inbound_Ipv4()
{
	static const GUID g =
	{
		0x86d07155,
		0x885f,
		0x409a,
		{ 0x8f, 0x22, 0x1, 0x9f, 0x87, 0x7a, 0xe4, 0x9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterBlockAll_Inbound_Ipv6()
{
	static const GUID g =
	{
		0x18b8c1d2,
		0x5910,
		0x4b51,
		{ 0xa5, 0x48, 0x1e, 0xfc, 0xd5, 0x4b, 0x63, 0xe9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_10_8()
{
	static const GUID g =
	{
		0x73fe6348,
		0x62f4,
		0x4686,
		{ 0x95, 0x47, 0x51, 0xa8, 0x21, 0xb, 0xa3, 0x8f }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_172_16_12()
{
	static const GUID g =
	{
		0x7a38dae,
		0x150f,
		0x47f1,
		{ 0xa6, 0xac, 0x99, 0x3, 0x48, 0x53, 0x83, 0x26 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_192_168_16()
{
	static const GUID g =
	{
		0x518bfc38,
		0xa7c5,
		0x42fe,
		{ 0xa3, 0xf2, 0xe1, 0x56, 0x24, 0xd7, 0x86, 0x1c }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_169_254_16()
{
	static const GUID g =
	{
		0x58718a9e,
		0x7ec1,
		0x4dee,
		{ 0x8d, 0x3f, 0x16, 0x5b, 0x95, 0x5d, 0xb5, 0x42 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Multicast()
{
	static const GUID g =
	{
		0xea5e136b,
		0xd951,
		0x4263,
		{ 0x99, 0xd8, 0x85, 0xc3, 0xf6, 0x4b, 0xda, 0xe9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Ipv6_fe80_10()
{
	static const GUID g =
	{
		0x5733b308,
		0x5856,
		0x469f,
		{ 0xa9, 0xf2, 0x24, 0x87, 0x52, 0x61, 0xd1, 0x6 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLan_Ipv6_Multicast()
{
	static const GUID g =
	{
		0x7379135f,
		0x6ce5,
		0x4107,
		{ 0x8a, 0x69, 0xf8, 0xea, 0x5a, 0x92, 0xb4, 0x97 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_10_8()
{
	static const GUID g =
	{
		0x24ed3b23,
		0x5d5a,
		0x4f1e,
		{ 0x8c, 0xfa, 0xfd, 0x68, 0x79, 0x6a, 0x83, 0x8a }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_172_16_12()
{
	static const GUID g =
	{
		0xa925dc62,
		0x54ea,
		0x46f5,
		{ 0x9d, 0x37, 0xa9, 0x5a, 0xf2, 0x84, 0xc3, 0x6f }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_192_168_16()
{
	static const GUID g =
	{
		0x97fd73cb,
		0x9bf0,
		0x47f2,
		{ 0x98, 0x69, 0xd1, 0x5e, 0xf3, 0x5c, 0x3a, 0x8 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_169_254_16()
{
	static const GUID g =
	{
		0x39d9b695,
		0x5c27,
		0x42a6,
		{ 0xba, 0xea, 0x8c, 0x4b, 0xe0, 0x7e, 0x66, 0x3e }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLanService_Ipv6_fe80_10()
{
	static const GUID g =
	{
		0xd1dff9da,
		0x1d12,
		0x4425,
		{ 0x82, 0x70, 0xdc, 0x7, 0x56, 0xff, 0xb9, 0xf2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xd9ff592d,
		0xbe46,
		0x49fb,
		{ 0x97, 0xec, 0x71, 0x1, 0x3c, 0x12, 0xb8, 0x30 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x764d4944,
		0x8a1e,
		0x4d96,
		{ 0xbf, 0xf0, 0x8d, 0xa6, 0x4f, 0x31, 0x44, 0xa2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Inbound_Ipv4()
{
	static const GUID g =
	{
		0xb8efb500,
		0xc51,
		0x4550,
		{ 0xbf, 0x5c, 0x48, 0x54, 0xa6, 0xc8, 0x48, 0xb9 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitLoopback_Inbound_Ipv6()
{
	static const GUID g =
	{
		0xbad325b0,
		0x736c,
		0x4e67,
		{ 0x8b, 0x37, 0x62, 0xb2, 0xdb, 0xe7, 0xd6, 0xeb }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpV4_Outbound_Request()
{
	static const GUID g =
	{
		0x6cf1687b,
		0x35e9,
		0x4d18,
		{ 0xa2, 0x3, 0xb2, 0x6b, 0x71, 0xa9, 0x5f, 0x8d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpV6_Outbound_Request()
{
	static const GUID g =
	{
		0x67bd69b0,
		0x522d,
		0x4631,
		{ 0x9a, 0x8f, 0x1c, 0xee, 0xdf, 0x64, 0xb7, 0x2b }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpV4_Inbound_Response()
{
	static const GUID g =
	{
		0x2db298d7,
		0x4108,
		0x47ff,
		{ 0x85, 0x99, 0xaf, 0xa5, 0xcb, 0x95, 0x9c, 0x25 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitDhcpV6_Inbound_Response()
{
	static const GUID g =
	{
		0x40dcfb6d,
		0x2ee,
		0x4531,
		{ 0x86, 0x61, 0xc4, 0xc8, 0xa4, 0x3a, 0xf4, 0x23 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnRelay()
{
	static const GUID g =
	{
		0x160c205d,
		0xdb40,
		0x4f79,
		{ 0x90, 0x6d, 0xfd, 0xa1, 0xe1, 0xc1, 0x8a, 0x70 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xdfdcbb76,
		0x2284,
		0x4b03,
		{ 0x93, 0x4e, 0x93, 0xe5, 0xd3, 0x84, 0x8c, 0xf1 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnel_Outbound_Ipv6()
{
	static const GUID g =
	{
		0x9b1fa7d,
		0x843b,
		0x4946,
		{ 0xa6, 0x2, 0x90, 0x4, 0x26, 0x2a, 0xb8, 0x6b }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Ipv4()
{
	static const GUID g =
	{
		0xc0792b44,
		0xfc3c,
		0x42e8,
		{ 0xa6, 0x60, 0x25, 0x4b, 0xd0, 0x4, 0xb1, 0x9d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Ipv6()
{
	static const GUID g =
	{
		0xcde477eb,
		0x2d8a,
		0x45b8,
		{ 0x9a, 0x3e, 0x9a, 0xa3, 0xbe, 0x4d, 0xe2, 0xb4 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv4()
{
	static const GUID g =
	{
		0x790445dc,
		0xb23e,
		0x4ab4,
		{ 0x8e, 0x2f, 0xc7, 0x6, 0x55, 0x5f, 0x94, 0xff }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterRestrictDns_Outbound_Tunnel_Ipv6()
{
	static const GUID g =
	{
		0xacc90d87,
		0xab77,
		0x4cf4,
		{ 0x84, 0xee, 0x1d, 0x68, 0x95, 0xf0, 0x66, 0xc2 }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnelService_Ipv4()
{
	static const GUID g =
	{
		0xf11a9ab4,
		0x3dd6,
		0x4cd9,
		{ 0x9d, 0x95, 0xb0, 0x36, 0x22, 0x71, 0x6b, 0x3d }
	};

	return g;
}

//static
const GUID &MullvadGuids::FilterPermitVpnTunnelService_Ipv6()
{
	static const GUID g =
	{
		0xe902e448,
		0x1845,
		0x42e5,
		{ 0xad, 0xf3, 0x33, 0xb2, 0x7a, 0xd, 0x5d, 0x38 }
	};

	return g;
}
