#include <iostream>
#include <vector>

#include <curl/curl.h>
#include <curl/multi.h>

int main(int argc, char * * argv) {
	if (argc < 2) {
		std::cerr << "Usage: " << argv[0] << " URL [URL ...]\n";
		return 1;
	}

	CURLM * multi = curl_multi_init();
	if (multi == nullptr) {
		std::cerr << "Failed to initialize CURL multi handle.\n";
		return 1;
	}

	std::vector<CURL *> handles;
	handles.reserve(argc - 1);

	for (int i = 1; i < argc; ++i) {
		CURL * handle = curl_easy_init();
		if (!handle) {
			std::cerr << "Failed to initialize CURL handle " << i << ".\n";
			return 1;
		}

		handles.push_back(handle);

		{
			CURLcode error = curl_easy_setopt(handle, CURLOPT_URL, argv[i]);
			if (error != CURLE_OK) {
				std::cerr << "Error " << error << " in curl_easy_setopt for handle " << i << ".\n";
				return 1;
			}
		}

		{
			CURLMcode error = curl_multi_add_handle(multi, handle);
			if (error != CURLM_OK) {
				std::cerr << "Error " << error << " in curl_multi_add_handle for handle " << i << ".\n";
			}
		}
	}

	int running = 0;
	do {
		CURLMcode error = curl_multi_perform(multi, &running);
		if (error != CURLM_OK) {
			std::cerr << "Error " << error << " in curl_multi_perform.\n";
		}
		std::cerr << "Running transfers: " << running << ".\n";
		if (running > 0) {
			curl_multi_poll(multi, nullptr, 0, 1000, nullptr);
		}
	} while (running != 0);

	for (auto handle : handles) {
		curl_multi_remove_handle(multi, handle);
		curl_easy_cleanup(handle);
	}

	curl_multi_cleanup(multi);
}
