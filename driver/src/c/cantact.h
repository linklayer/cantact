#ifndef CANTACT_H_
#define CANTACT_H_

#include <stdint.h>

typedef void* cantacthnd;

struct CantactFrame {
	uint8_t channel;
	uint32_t id;
	uint8_t dlc;
	uint8_t data[8];
	uint8_t ext;
	uint8_t fd;
	uint8_t loopback;
	uint8_t rtr;
};

extern "C" {
	__declspec(dllimport) cantacthnd cantact_init();
	__declspec(dllimport) int32_t cantact_deinit(cantacthnd hnd);

	__declspec(dllimport) int32_t cantact_open(cantacthnd hnd);
	__declspec(dllimport) int32_t cantact_close(cantacthnd hnd);

	__declspec(dllimport) int32_t cantact_set_rx_callback(cantacthnd hnd, void(__cdecl* callback)(CantactFrame* f));

	__declspec(dllimport) int32_t cantact_start(cantacthnd hnd);
	__declspec(dllimport) int32_t cantact_stop(cantacthnd hnd);

	__declspec(dllimport) int32_t cantact_transmit(cantacthnd hnd, const struct CantactFrame f);

	__declspec(dllimport) int32_t cantact_set_bitrate(cantacthnd hnd, uint8_t channel, uint32_t bitrate);
	__declspec(dllimport) int32_t cantact_set_enabled(cantacthnd hnd, uint8_t channel, uint8_t enabled);
	__declspec(dllimport) int32_t cantact_set_monitor(cantacthnd hnd, uint8_t channel, uint8_t enabled);
	__declspec(dllimport) int32_t cantact_set_hw_loopback(cantacthnd hnd, uint8_t channel, uint8_t enabled);

	__declspec(dllimport) int32_t cantact_get_channel_count(cantacthnd hnd);
}

#endif