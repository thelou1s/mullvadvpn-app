package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

class RelayList {
    val countries: List<RelayCountry>

    constructor(model: net.mullvad.mullvadvpn.model.RelayList) {
        countries = model.countries.map { country ->
            val cities = country.cities.map { city -> 
                val relays = city.relays.map { relay -> Relay(relay.hostname) }

                RelayCity(city.name, city.code, false, relays)
            }

            RelayCountry(country.name, country.code, false, cities)
        }
    }

    fun findItemForLocation(location: LocationConstraint?): RelayItem? {
        when (location) {
            null -> return null
            is LocationConstraint.Country -> {
                return countries.find { country -> country.code == location.countryCode }
            }
            is LocationConstraint.City -> {
                val country = countries.find { country -> country.code == location.countryCode }

                return country?.cities?.find { city -> city.code == location.cityCode }
            }
            is LocationConstraint.Hostname -> {
                val country = countries.find { country -> country.code == location.countryCode }
                val city = country?.cities?.find { city -> city.code == location.cityCode }

                return city?.relays?.find { relay -> relay.name == location.hostname }
            }
        }
    }
}
